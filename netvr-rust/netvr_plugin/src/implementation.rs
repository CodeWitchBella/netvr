use anyhow::{anyhow, Result};
use netvr_data::{InstanceAndSession, Nothing, RemoteDevice};
use tokio::select;
use tracing::info;
use xr_layer::log::LogInfo;

use crate::{net_client::run_net_client, overrides::with_layer};

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
        for (client_id, client) in state.clients.iter() {
            let mut i = 0;
            for device in client.views.iter() {
                i += 1;
                devices.devices.push(RemoteDevice {
                    id: client_id * 100 + i,
                    pos: device.position.clone(),
                    rot: device.orientation.clone(),
                });
            }
            for device in client.controllers.iter() {
                i += 1;
                devices.devices.push(RemoteDevice {
                    id: client_id * 100 + i,
                    pos: device.position.clone(),
                    rot: device.orientation.clone(),
                });
            }
        }
        Ok(devices)
    })
}
