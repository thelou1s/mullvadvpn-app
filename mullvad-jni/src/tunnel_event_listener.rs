use crate::{get_class, into_java::IntoJava, lock_ipc_client};
use error_chain::*;
use futures::{stream::Wait, Future, Stream};
use jni::{
    objects::{JMethodID, JObject, JValue},
    signature::{JavaType, Primitive},
    AttachGuard, JNIEnv,
};
use jsonrpc_client_pubsub::Subscription;
use std::thread;
use talpid_types::tunnel::TunnelStateTransition;

error_chain! {
    errors {
        CreateGlobalReference {
            description("Failed to create global reference to MullvadIpcClient Java object")
        }

        GetJvmInstance {
            description("Failed to retrieve Java VM instance")
        }
    }
}

pub struct TunnelEventListener<'env> {
    env: AttachGuard<'env>,
    mullvad_ipc_client: JObject<'env>,
    notify_tunnel_event: JMethodID<'env>,
    subscription: Wait<Subscription<TunnelStateTransition>>,
}

impl TunnelEventListener<'_> {
    pub fn spawn(old_env: &JNIEnv, old_mullvad_ipc_client: &JObject) -> Result<()> {
        let jvm = old_env
            .get_java_vm()
            .chain_err(|| ErrorKind::GetJvmInstance)?;
        let mullvad_ipc_client = old_env
            .new_global_ref(*old_mullvad_ipc_client)
            .chain_err(|| ErrorKind::CreateGlobalReference)?;

        thread::spawn(move || match jvm.attach_current_thread() {
            Ok(env) => TunnelEventListener::run(env, mullvad_ipc_client.as_obj()),
            Err(error) => {
                let chained_error =
                    error.chain_err(|| "Failed to attach tunnel event listener thread to Java VM");
                log::error!("{}", chained_error.display_chain());
            }
        });

        Ok(())
    }
}

impl<'env> TunnelEventListener<'env> {
    fn run(env: AttachGuard<'env>, mullvad_ipc_client: JObject<'env>) {
        match Self::subscribe() {
            Ok(subscription) => match Self::new(env, mullvad_ipc_client, subscription) {
                Ok(mut listener) => listener.handle_events(),
                Err(error) => {
                    let chained_error =
                        error.chain_err(|| "Failed to create tunnel event listener");
                    log::error!("{}", chained_error.display_chain());
                }
            },
            Err(error) => {
                let chained_error = error.chain_err(|| "Failed to subscribe to tunnel events");
                log::error!("{}", chained_error.display_chain());
            }
        }
    }

    fn subscribe() -> Result<Subscription<TunnelStateTransition>> {
        let mut ipc_client = lock_ipc_client();

        ipc_client
            .new_state_subscribe()
            .wait()
            .chain_err(|| "Subscription request failed")
    }
}

impl<'env> TunnelEventListener<'env> {
    fn new(
        env: AttachGuard<'env>,
        mullvad_ipc_client: JObject<'env>,
        subscription: Subscription<TunnelStateTransition>,
    ) -> Result<Self> {
        let class = get_class("net/mullvad/mullvadvpn/MullvadIpcClient");
        let notify_tunnel_event = env
            .get_method_id(
                &class,
                "notifyTunnelStateEvent",
                "(Lnet/mullvad/mullvadvpn/model/TunnelStateTransition;)V",
            )
            .chain_err(|| "Failed to find notifyTunnelStateEvent method")?;

        Ok(TunnelEventListener {
            env,
            mullvad_ipc_client,
            notify_tunnel_event,
            subscription: subscription.wait(),
        })
    }

    fn handle_events(&mut self) {
        while let Some(event_result) = self.subscription.next() {
            match event_result {
                Ok(event) => self.handle_event(event),
                Err(error) => {
                    let chained_error = error.chain_err(|| "Failed to retrieve tunnel state event");
                    log::error!("{}", chained_error.display_chain());
                }
            }
        }
    }

    fn handle_event(&self, event: TunnelStateTransition) {
        let result = self
            .env
            .call_method_unchecked(
                self.mullvad_ipc_client,
                self.notify_tunnel_event,
                JavaType::Primitive(Primitive::Void),
                &[JValue::Object(event.into_java(&self.env))],
            )
            .chain_err(|| "Failed to call MullvadIpcClient.notifyTunnelStateEvent");

        if let Err(error) = result {
            log::error!("{}", error.display_chain());
        }
    }
}
