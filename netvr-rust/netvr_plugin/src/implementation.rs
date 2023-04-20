use anyhow::{anyhow, Result};
use netvr_data::{InstanceAndSession, Nothing};
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
        // TODO: use state instead of the following
        // TODO: remove instance.views as it is not read anymore
        let vec = instance
            .views
            .lock()
            .map_err(|err| anyhow!("Failed to acquire view lock {:?}", err))?;
        let mut i = 0;
        for v in vec.iter() {
            i += 1;
            let device = netvr_data::RemoteDevice {
                id: i.try_into().unwrap(),
                pos: v.pose.position.into(),
                rot: v.pose.orientation.into(),
            };
            devices.devices.push(device);
        }
        Ok(devices)
    })
}

pub(crate) fn post_poll_event(
    _instance: &Instance,
) -> Result<Option<EventDataBuffer>, XrWrapError> {
    Ok(None)
}
