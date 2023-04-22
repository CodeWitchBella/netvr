use anyhow::{anyhow, Result};
use netvr_data::{InstanceAndSession, Nothing, RemoteDevice};
use tokio::select;
use tracing::info;
use xr_layer::{log::LogInfo, EventDataBuffer};

use crate::{
    instance::Instance, net_client::run_net_client, overrides::with_layer, xr_wrap::XrWrapError,
};

/// Starts the netvr client. Should be called after xrCreateInstance.
pub(crate) fn start(input: InstanceAndSession) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        info!("start {:?}", instance.instance.as_raw());

        let token = instance.token.clone();
        instance.tokio.spawn(async move {
            loop {
                select! {
                    _ = token.cancelled() => { break; }
                    res = run_net_client(input.instance, input.session) => {
                        LogInfo::string(format!("net_client finished {:?}", res));
                    }
                }
            }
        });

        Ok(Nothing::default())
    })
}

pub(crate) fn read_remote_devices(
    input: InstanceAndSession,
) -> Result<netvr_data::ReadRemoteDevicesOutput> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let mut devices = netvr_data::ReadRemoteDevicesOutput::default();
        let state = session.remote_state.read().map_err(|err| {
            anyhow::anyhow!("Failed to acquire read lock for remote_state: {:?}", err)
        })?;
        for client in state.clients.iter() {
            let mut i = 0;
            for device in client.1.views.iter() {
                i += 1;
                devices.devices.push(RemoteDevice {
                    id: client.0 * 100 + i,
                    pos: device.position.clone(),
                    rot: device.orientation.clone(),
                });
            }
        }

        // TODO: remove instance.views as it is not read anymore
        Ok(devices)
    })
}
