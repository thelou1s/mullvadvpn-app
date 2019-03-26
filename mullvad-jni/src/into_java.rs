use super::get_class;
use jni::{
    objects::{JList, JObject, JString, JValue},
    sys::jint,
    JNIEnv,
};
use mullvad_types::{
    account::AccountData,
    relay_constraints::{Constraint, LocationConstraint, RelayConstraints, RelaySettings},
    relay_list::{Relay, RelayList, RelayListCity, RelayListCountry},
    settings::Settings,
    CustomTunnelEndpoint,
};
use std::fmt::Debug;
use talpid_types::tunnel::TunnelStateTransition;

pub trait IntoJava<'env> {
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
            "(Lnet/mullvad/mullvadvpn/model/Constraint;)V",
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
            Constraint::Any => {
                let class = get_class("net/mullvad/mullvadvpn/model/Constraint$Any");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create Constraint.Any Java object");
                        JObject::null()
                    }
                }
            }
            Constraint::Only(constraint) => {
                let class = get_class("net/mullvad/mullvadvpn/model/Constraint$Only");
                let value = JObject::from(constraint.into_java(env));
                let parameters = [JValue::Object(value)];

                let result = env.new_object(&class, "(Ljava/lang/Object;)V", &parameters);

                let _ = env.delete_local_ref(value);

                match result {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create Constraint.Only Java object");
                        JObject::null()
                    }
                }
            }
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

impl<'env> IntoJava<'env> for TunnelStateTransition {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            TunnelStateTransition::Disconnected => {
                let class =
                    get_class("net/mullvad/mullvadvpn/model/TunnelStateTransition$Disconnected");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!(
                            "Failed to create TunnelStateTransition.Disconnected Java object"
                        );
                        JObject::null()
                    }
                }
            }
            TunnelStateTransition::Connecting(_) => {
                let class =
                    get_class("net/mullvad/mullvadvpn/model/TunnelStateTransition$Connecting");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!(
                            "Failed to create TunnelStateTransition.Connecting Java object"
                        );
                        JObject::null()
                    }
                }
            }
            TunnelStateTransition::Connected(_) => {
                let class =
                    get_class("net/mullvad/mullvadvpn/model/TunnelStateTransition$Connected");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create TunnelStateTransition.Connected Java object");
                        JObject::null()
                    }
                }
            }
            TunnelStateTransition::Disconnecting(_) => {
                let class =
                    get_class("net/mullvad/mullvadvpn/model/TunnelStateTransition$Disconnecting");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!(
                            "Failed to create TunnelStateTransition.Disconnecting Java object"
                        );
                        JObject::null()
                    }
                }
            }
            TunnelStateTransition::Blocked(_) => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelStateTransition$Blocked");

                match env.new_object(&class, "()V", &[]) {
                    Ok(object) => object,
                    Err(_) => {
                        log::error!("Failed to create TunnelStateTransition.Blocked Java object");
                        JObject::null()
                    }
                }
            }
        }
    }
}
