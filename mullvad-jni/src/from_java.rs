use crate::{get_class, is_null::IsNull};
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
use mullvad_types::{
    relay_constraints::{
        Constraint, LocationConstraint, RelayConstraintsUpdate, RelaySettingsUpdate,
    },
    CustomTunnelEndpoint,
};
use std::fmt::Debug;

pub trait FromJava<'env> {
    type JavaType: 'env;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self;
}

impl<'env, T> FromJava<'env> for Option<T>
where
    T: FromJava<'env>,
    T::JavaType: IsNull,
{
    type JavaType = T::JavaType;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        if source.is_null() {
            None
        } else {
            Some(T::from_java(env, source))
        }
    }
}

impl<'env> FromJava<'env> for String {
    type JavaType = JString<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        String::from(
            env.get_string(source)
                .expect("Failed to convert from Java String"),
        )
    }
}

impl<'env> FromJava<'env> for RelaySettingsUpdate {
    type JavaType = JObject<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        let custom_tunnel_endpoint =
            get_class("net/mullvad/mullvadvpn/model/RelaySettingsUpdate$CustomTunnelEndpoint");
        let relay_constraints_update =
            get_class("net/mullvad/mullvadvpn/model/RelaySettingsUpdate$RelayConstraintsUpdate");

        if env.is_instance_of(source, &custom_tunnel_endpoint).unwrap() {
            RelaySettingsUpdate::CustomTunnelEndpoint(CustomTunnelEndpoint::from_java(env, source))
        } else if env
            .is_instance_of(source, &relay_constraints_update)
            .unwrap()
        {
            RelaySettingsUpdate::Normal(RelayConstraintsUpdate::from_java(env, source))
        } else {
            panic!("Invalid RelaySettingsUpdate Java sub-class");
        }
    }
}

impl<'env> FromJava<'env> for CustomTunnelEndpoint {
    type JavaType = JObject<'env>;

    fn from_java(_env: &JNIEnv<'env>, _source: Self::JavaType) -> Self {
        unimplemented!();
    }
}

impl<'env> FromJava<'env> for RelayConstraintsUpdate {
    type JavaType = JObject<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        let location = env
            .get_field(
                source,
                "location",
                "Lnet/mullvad/mullvadvpn/model/Constraint;",
            )
            .unwrap()
            .l()
            .unwrap();

        RelayConstraintsUpdate {
            location: FromJava::from_java(env, location),
            tunnel: None,
        }
    }
}

impl<'env, T> FromJava<'env> for Constraint<T>
where
    T: Clone + Debug + Eq + FromJava<'env>,
    T::JavaType: From<JObject<'env>>,
{
    type JavaType = JObject<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        let any = get_class("net/mullvad/mullvadvpn/model/Constraint$Any");
        let only = get_class("net/mullvad/mullvadvpn/model/Constraint$Only");

        if env.is_instance_of(source, &any).unwrap() {
            Constraint::Any
        } else if env.is_instance_of(source, &only).unwrap() {
            let value = env
                .get_field(source, "value", "Ljava/lang/Object;")
                .unwrap()
                .l()
                .unwrap();

            Constraint::Only(T::from_java(env, T::JavaType::from(value)))
        } else {
            panic!("Invalid Constraint Java sub-class");
        }
    }
}

impl<'env> FromJava<'env> for LocationConstraint {
    type JavaType = JObject<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        let country_class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Country");
        let city_class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$City");
        let hostname_class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Hostname");

        if env.is_instance_of(source, &country_class).unwrap() {
            let country = JString::from(
                env.get_field(source, "countryCode", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );

            LocationConstraint::Country(String::from_java(env, country))
        } else if env.is_instance_of(source, &city_class).unwrap() {
            let country = JString::from(
                env.get_field(source, "countryCode", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );
            let city = JString::from(
                env.get_field(source, "cityCode", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );

            LocationConstraint::City(
                String::from_java(env, country),
                String::from_java(env, city),
            )
        } else if env.is_instance_of(source, &hostname_class).unwrap() {
            let country = JString::from(
                env.get_field(source, "countryCode", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );
            let city = JString::from(
                env.get_field(source, "cityCode", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );
            let hostname = JString::from(
                env.get_field(source, "hostname", "Ljava/lang/String;")
                    .unwrap()
                    .l()
                    .unwrap(),
            );

            LocationConstraint::Hostname(
                String::from_java(env, country),
                String::from_java(env, city),
                String::from_java(env, hostname),
            )
        } else {
            panic!("Invalid LocationConstraint Java sub-class");
        }
    }
}
