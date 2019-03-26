mod from_java;
mod into_java;
mod is_null;

use crate::{from_java::FromJava, into_java::IntoJava};
use error_chain::*;
use jni::{
    objects::{GlobalRef, JObject, JString},
    JNIEnv,
};
use lazy_static::lazy_static;
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use mullvad_paths::{get_log_dir, get_rpc_socket_path};
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, RwLock},
    thread,
    time::Duration,
};

static CLASSES_TO_LOAD: &[&str] = &[
    "java/util/ArrayList",
    "net/mullvad/mullvadvpn/model/AccountData",
    "net/mullvad/mullvadvpn/model/Constraint$Any",
    "net/mullvad/mullvadvpn/model/Constraint$Only",
    "net/mullvad/mullvadvpn/model/LocationConstraint$City",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Country",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Hostname",
    "net/mullvad/mullvadvpn/model/Relay",
    "net/mullvad/mullvadvpn/model/RelayList",
    "net/mullvad/mullvadvpn/model/RelayListCity",
    "net/mullvad/mullvadvpn/model/RelayListCountry",
    "net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$RelayConstraintsUpdate",
    "net/mullvad/mullvadvpn/model/Settings",
];

lazy_static! {
    static ref IPC_CLIENT: Mutex<DaemonRpcClient> = connect();
    static ref CLASSES: RwLock<HashMap<&'static str, GlobalRef>> =
        RwLock::new(HashMap::with_capacity(CLASSES_TO_LOAD.len()));
}

error_chain! {}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_loadClasses(
    env: JNIEnv,
    _: JObject,
) {
    let mut classes = CLASSES.write().unwrap();

    for class in CLASSES_TO_LOAD {
        classes.insert(class, load_class_reference(&env, class));
    }
}

fn load_class_reference(env: &JNIEnv, name: &str) -> GlobalRef {
    let class = match env.find_class(name) {
        Ok(class) => class,
        Err(_) => {
            log::error!("Failed to find {} Java class", name);
            panic!("Missing class");
        }
    };

    let global_class_ref = match env.new_global_ref(JObject::from(class)) {
        Ok(global_ref) => global_ref,
        Err(_) => {
            log::error!(
                "Failed to convert local reference to {} Java class into a global reference",
                name
            );
            panic!("Failed to convert local ref to global ref");
        }
    };

    global_class_ref
}

pub fn get_class(name: &str) -> GlobalRef {
    match CLASSES.read().unwrap().get(name) {
        Some(class) => class.clone(),
        None => {
            log::error!("Class not loaded: {}", name);
            panic!("Missing class");
        }
    }
}

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
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_connect(_: JNIEnv, _: JObject) {
    let mut ipc_client = lock_ipc_client();

    if let Err(error) = ipc_client.connect() {
        let chained_error = error.chain_err(|| "Failed to request daemon to connect");
        log::error!("{}", chained_error.display_chain());
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_getAccountData<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
    accountToken: JString,
) -> JObject<'env> {
    let mut ipc_client = lock_ipc_client();

    let account = String::from_java(&env, accountToken);

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
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_getRelayLocations<
    'env,
    'this,
>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let mut ipc_client = lock_ipc_client();

    match ipc_client.get_relay_locations() {
        Ok(relay_list) => relay_list.into_java(&env),
        Err(error) => {
            let chained_error = error.chain_err(|| "Failed to get relay locations");
            log::error!("{}", chained_error.display_chain());
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_getSettings<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    let mut ipc_client = lock_ipc_client();

    match ipc_client.get_settings() {
        Ok(settings) => settings.into_java(&env),
        Err(error) => {
            let chained_error = error.chain_err(|| "Failed to get settings");
            log::error!("{}", chained_error.display_chain());
            JObject::null()
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

    let account = <Option<String> as FromJava>::from_java(&env, accountToken);

    if let Err(error) = ipc_client.set_account(account) {
        let chained_error = error.chain_err(|| "Failed to set account");
        log::error!("{}", chained_error.display_chain());
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadIpcClient_updateRelaySettings(
    env: JNIEnv,
    _: JObject,
    relaySettingsUpdate: JObject,
) {
    let mut ipc_client = lock_ipc_client();
    let update = FromJava::from_java(&env, relaySettingsUpdate);

    if let Err(error) = ipc_client.update_relay_settings(update) {
        let chained_error = error.chain_err(|| "Failed to update relay settings");
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
