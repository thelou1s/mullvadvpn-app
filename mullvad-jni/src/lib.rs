use error_chain::*;
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};
use lazy_static::lazy_static;
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use mullvad_paths::{get_log_dir, get_rpc_socket_path};
use mullvad_types::{account::AccountData, settings::Settings};
use std::{
    ptr,
    sync::{Mutex, MutexGuard},
    thread,
    time::Duration,
};

lazy_static! {
    static ref IPC_CLIENT: Mutex<DaemonRpcClient> = connect();
}

error_chain! {}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_startLogging(
    _: JNIEnv,
    _: JObject,
) {
    let log_file = get_log_dir()
        .expect("Failed to get log directory")
        .join("jni.log");

    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(
            fern::log_file(&log_file)
                .expect(&format!("Failed to log to file: {}", log_file.display())),
        )
        .apply();
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_getAccountData<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
    accountToken: JString,
) -> JObject<'env> {
    let mut ipc_client = lock_ipc_client();

    let account = if accountToken.into_inner() == ptr::null_mut() {
        log::error!("Attempt to get account data for no account");
        return JObject::null();
    } else {
        String::from(
            env.get_string(accountToken)
                .expect("Failed to convert account string from Java type"),
        )
    };

    match ipc_client.get_account_data(account) {
        Ok(data) => data.into_java(&env),
        Err(error) => {
            let chained_error = error.chain_err(|| "Failed to get account data");
            log::error!("{}", chained_error.display_chain());
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_getCurrentVersion<
    'env,
    'this,
>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JString<'env> {
    let mut ipc_client = lock_ipc_client();

    match ipc_client.get_current_version() {
        Ok(version) => version.into_java(&env),
        Err(error) => {
            let chained_error = error.chain_err(|| "Failed to get current version");
            log::error!("{}", chained_error.display_chain());
            JString::from(JObject::null())
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_setAccount(
    env: JNIEnv,
    _: JObject,
    accountToken: JString,
) {
    let mut ipc_client = lock_ipc_client();

    let account = if accountToken.into_inner() == ptr::null_mut() {
        None
    } else {
        Some(String::from(
            env.get_string(accountToken)
                .expect("Failed to convert account string from Java type"),
        ))
    };

    if let Err(error) = ipc_client.set_account(account) {
        let chained_error = error.chain_err(|| "Failed to set account");
        log::error!("{}", chained_error.display_chain());
    }
}

fn connect() -> Mutex<DaemonRpcClient> {
    for attempt in 1..=10 {
        log::debug!("Connection attempt {}", attempt);

        match try_connect() {
            Ok(ipc_client) => return Mutex::new(ipc_client),
            Err(error) => log::warn!("{}", error.display_chain()),
        }

        let delay = (attempt - 1) * 50;
        log::warn!("Retrying in {} ms", delay);
        thread::sleep(Duration::from_millis(delay));
    }

    log::error!("Failed to connect to daemon");
    panic!();
}

fn try_connect() -> Result<DaemonRpcClient> {
    let rpc_socket_path = get_rpc_socket_path();

    log::debug!(
        "Connecting to daemon through socket: {}",
        rpc_socket_path.display()
    );

    new_standalone_ipc_client(&rpc_socket_path).chain_err(|| "Failed to initialize IPC client")
}

fn lock_ipc_client() -> MutexGuard<'static, DaemonRpcClient> {
    IPC_CLIENT.lock().expect("IPC client mutex was poisoned")
}

trait IntoJava<'env> {
    type JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType;
}

impl<'env> IntoJava<'env> for String {
    type JavaType = JString<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match env.new_string(&self) {
            Ok(string) => string,
            Err(_) => {
                log::error!(r#"Failed to create Java String from "{}""#, self);
                JString::from(JObject::null())
            }
        }
    }
}

impl<'env> IntoJava<'env> for AccountData {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = match env.find_class("net/mullvad/mullvadvpn/AccountData") {
            Ok(class) => class,
            Err(_) => {
                log::error!("Failed to find AccountData Java class");
                return JObject::null();
            }
        };

        let account_expiry = self.expiry.to_string().into_java(env);
        let parameters = [JValue::Object(JObject::from(account_expiry))];

        match env.new_object(class, "(Ljava/lang/String;)V", &parameters) {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create AccountData Java object");
                JObject::null()
            }
        }
    }
}
