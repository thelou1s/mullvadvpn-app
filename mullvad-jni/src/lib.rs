use error_chain::*;
use jni::{
    objects::{GlobalRef, JList, JObject, JString, JValue},
    sys::jint,
    JNIEnv,
};
use lazy_static::lazy_static;
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use mullvad_paths::{get_log_dir, get_rpc_socket_path};
use mullvad_types::{
    account::AccountData,
    relay_constraints::{Constraint, LocationConstraint, RelayConstraints, RelaySettings},
    relay_list::{Relay, RelayList, RelayListCity, RelayListCountry},
    settings::Settings,
    CustomTunnelEndpoint,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    ptr,
    sync::{Mutex, MutexGuard, RwLock},
    thread,
    time::Duration,
};

static CLASSES_TO_LOAD: &[&str] = &[
    "java/util/ArrayList",
    "net/mullvad/mullvadvpn/model/AccountData",
    "net/mullvad/mullvadvpn/model/LocationConstraint$City",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Country",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Hostname",
    "net/mullvad/mullvadvpn/model/Relay",
    "net/mullvad/mullvadvpn/model/RelayList",
    "net/mullvad/mullvadvpn/model/RelayListCity",
    "net/mullvad/mullvadvpn/model/RelayListCountry",
    "net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints",
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

fn get_class(name: &str) -> GlobalRef {
    match CLASSES.read().unwrap().get(name) {
        Some(class) => class.clone(),
        None => {
            log::error!("Class not loaded: {}", name);
            panic!("Missing class");
        }
    }
}

fn lock_ipc_client() -> MutexGuard<'static, DaemonRpcClient> {
    IPC_CLIENT.lock().expect("IPC client mutex was poisoned")
}

trait IntoJava<'env> {
    type JavaType: 'env;

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

impl<'env, T> IntoJava<'env> for Option<T>
where
    T: IntoJava<'env>,
    T::JavaType: From<JObject<'env>>,
{
    type JavaType = T::JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            Some(data) => data.into_java(env),
            None => T::JavaType::from(JObject::null()),
        }
    }
}

impl<'env, T> IntoJava<'env> for Vec<T>
where
    T: IntoJava<'env>,
    JObject<'env>: From<T::JavaType>,
{
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("java/util/ArrayList");
        let initial_capacity = self.len();
        let parameters = [JValue::Int(initial_capacity as jint)];

        let list_object = match env.new_object(&class, "(I)V", &parameters) {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create ArrayList object");
                panic!("Failed to create ArrayList");
            }
        };

        let list = match JList::from_env(env, list_object) {
            Ok(list) => list,
            Err(error) => {
                log::error!("Failed to create List with ArrayList: {}", error);
                panic!("Failed to create JList");
            }
        };

        for element in self {
            let java_element = JObject::from(element.into_java(env));
            let _ = list.add(java_element);
            let _ = env.delete_local_ref(java_element);
        }

        list_object
    }
}

impl<'env> IntoJava<'env> for AccountData {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/AccountData");
        let account_expiry = JObject::from(self.expiry.to_string().into_java(env));
        let parameters = [JValue::Object(account_expiry)];

        let result = env.new_object(&class, "(Ljava/lang/String;)V", &parameters);

        let _ = env.delete_local_ref(account_expiry);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create AccountData Java object");
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelayList {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayList");
        let relay_countries = self.countries.into_java(env);
        let parameters = [JValue::Object(relay_countries)];

        let result = env.new_object(&class, "(Ljava/util/List;)V", &parameters);

        let _ = env.delete_local_ref(relay_countries);

        match result {
            Ok(object) => object,
            Err(error) => {
                log::error!("Failed to create RelayList Java object: {}", error);
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelayListCountry {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCountry");
        let name = JObject::from(self.name.into_java(env));
        let code = JObject::from(self.code.into_java(env));
        let relay_cities = self.cities.into_java(env);
        let parameters = [
            JValue::Object(name),
            JValue::Object(code),
            JValue::Object(relay_cities),
        ];

        let result = env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        );

        let _ = env.delete_local_ref(name);
        let _ = env.delete_local_ref(code);
        let _ = env.delete_local_ref(relay_cities);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create RelayListCountry Java object");
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelayListCity {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCity");
        let name = JObject::from(self.name.into_java(env));
        let code = JObject::from(self.code.into_java(env));
        let relays = self.relays.into_java(env);
        let parameters = [
            JValue::Object(name),
            JValue::Object(code),
            JValue::Object(relays),
        ];

        let result = env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        );

        let _ = env.delete_local_ref(name);
        let _ = env.delete_local_ref(code);
        let _ = env.delete_local_ref(relays);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create RelayListCity Java object");
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for Relay {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Relay");
        let hostname = JObject::from(self.hostname.into_java(env));
        let parameters = [JValue::Object(hostname)];

        let result = env.new_object(&class, "(Ljava/lang/String;)V", &parameters);

        let _ = env.delete_local_ref(hostname);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create Relay Java object");
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelaySettings {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            RelaySettings::CustomTunnelEndpoint(endpoint) => endpoint.into_java(env),
            RelaySettings::Normal(relay_constraints) => relay_constraints.into_java(env),
        }
    }
}

impl<'env> IntoJava<'env> for CustomTunnelEndpoint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint");

        match env.new_object(&class, "()V", &[]) {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create CustomTunnelEndpoint Java object");
                JObject::null()
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelayConstraints {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints");
        let location = self.location.into_java(env);
        let parameters = [JValue::Object(location)];

        let result = env.new_object(
            &class,
            "(Lnet/mullvad/mullvadvpn/model/LocationConstraint;)V",
            &parameters,
        );

        let _ = env.delete_local_ref(location);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create RelaySettings.RelayConstraints Java object");
                JObject::null()
            }
        }
    }
}

impl<'env, T> IntoJava<'env> for Constraint<T>
where
    T: Clone + Eq + Debug + IntoJava<'env>,
    JObject<'env>: From<T::JavaType>,
{
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            Constraint::Any => JObject::null(),
            Constraint::Only(constraint) => JObject::from(constraint.into_java(env)),
        }
    }
}

impl<'env> IntoJava<'env> for LocationConstraint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            LocationConstraint::Country(country_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Country");
                let country = JObject::from(country_code.into_java(env));
                let parameters = [JValue::Object(country)];

                let result = env.new_object(&class, "(Ljava/lang/String;)V", &parameters);

                let _ = env.delete_local_ref(country);

                match result {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create LocationConstraint.Country Java object");
                        JObject::null()
                    }
                }
            }
            LocationConstraint::City(country_code, city_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$City");
                let country = JObject::from(country_code.into_java(env));
                let city = JObject::from(city_code.into_java(env));
                let parameters = [JValue::Object(country), JValue::Object(city)];

                let result = env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                );

                let _ = env.delete_local_ref(country);
                let _ = env.delete_local_ref(city);

                match result {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create LocationConstraint.City Java object");
                        JObject::null()
                    }
                }
            }
            LocationConstraint::Hostname(country_code, city_code, hostname) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Hostname");
                let country = JObject::from(country_code.into_java(env));
                let city = JObject::from(city_code.into_java(env));
                let hostname = JObject::from(hostname.into_java(env));
                let parameters = [
                    JValue::Object(country),
                    JValue::Object(city),
                    JValue::Object(hostname),
                ];

                let result = env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                );

                let _ = env.delete_local_ref(country);
                let _ = env.delete_local_ref(city);
                let _ = env.delete_local_ref(hostname);

                match result {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create LocationConstraint.Hostname Java object");
                        JObject::null()
                    }
                }
            }
        }
    }
}

impl<'env> IntoJava<'env> for Settings {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Settings");
        let account_token = JObject::from(self.get_account_token().into_java(env));
        let relay_settings = self.get_relay_settings().into_java(env);
        let parameters = [
            JValue::Object(account_token),
            JValue::Object(relay_settings),
        ];

        let result = env.new_object(
            &class,
            "(Ljava/lang/String;Lnet/mullvad/mullvadvpn/model/RelaySettings;)V",
            &parameters,
        );

        let _ = env.delete_local_ref(account_token);

        match result {
            Ok(object) => object,
            Err(_) => {
                log::error!("Failed to create Settings Java object");
                JObject::null()
            }
        }
    }
}
