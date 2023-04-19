use netvr_data::{InstanceAndSession, JustInstance, Nothing};
use tokio::select;
use tracing::info;
use xr_layer::{log::LogInfo, EventDataBuffer};

use crate::{
    instance::Instance, net_client::run_net_client, overrides::with_layer, xr_wrap::XrWrapError,
};

/// Starts the netvr client. Should be called after xrCreateInstance.
pub(crate) fn start(input: InstanceAndSession) -> Result<Nothing, XrWrapError> {
    with_layer(input.instance, |instance| {
        info!("start {:?}", instance.instance.as_raw());

        let token = instance.token.clone();
        instance.tokio.spawn(async move {
            loop {
                select! {
                    _ = token.cancelled() => { break; }
                    _ = run_net_client(input.instance, input.session) => {
                        LogInfo::str("net_client finished");
                    }
                }
            }
        });

        Ok(Nothing::default())
    })
}

pub(crate) fn read_remote_devices(
    input: JustInstance,
) -> Result<netvr_data::ReadRemoteDevicesOutput, XrWrapError> {
    with_layer(input.instance, |instance| {
        let mut devices = netvr_data::ReadRemoteDevicesOutput::default();
        let vec = instance.views.lock()?;
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
