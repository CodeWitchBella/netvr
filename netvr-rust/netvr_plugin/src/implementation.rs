use netvr_data::{JustInstance, Nothing};
use tokio::select;
use tracing::{info, instrument};
use xr_layer::{
    log::{LogError, LogInfo},
    EventDataBuffer, XrDebug,
};

use crate::{
    instance::{Instance, Session},
    net_client::run_net_client,
    overrides::with_layer,
    xr_wrap::XrWrapError,
};

/// Starts the netvr client. Should be called after xrCreateInstance.
pub(crate) fn start(input: JustInstance) -> Result<Nothing, XrWrapError> {
    with_layer(input.instance, |instance| {
        info!("start {:?}", instance.instance.as_raw());

        let token = instance.token.clone();
        instance.tokio.spawn(async move {
            loop {
                select! {
                    _ = token.cancelled() => { break; }
                    _ = run_net_client(input.instance) => {
                        LogInfo::str("net_client finished");
                    }
                }
            }
        });

        Ok(Nothing::default())
    })
}

/// Should be periodically called from application. Sends data to network.
pub(crate) fn tick(input: JustInstance) -> Result<Nothing, XrWrapError> {
    with_layer(input.instance, |instance| {
        info!("tick {:?}", instance.instance.as_raw());

        for session in instance.sessions.values() {
            if let Err(error) = tick_session(instance, session) {
                LogError::string(format!("session_tick failed with: {:?}", error));
            }
        }
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

/// Called for each session once per tick.
#[instrument]
fn tick_session(instance: &Instance, session: &Session) -> Result<(), XrWrapError> {
    let r = session.read_space()?;
    let space = if let Some(val) = &*r {
        val
    } else {
        return Ok(());
    };
    let time = session.time;
    if time.as_nanos() < 0 {
        return Ok(());
    }

    let (info, views) =
        session
            .session
            .locate_views(session.view_configuration_type, session.time, space)?;
    info!(info = ?info, "views");
    for view in views {
        info!(view = ?view.pose.as_debug(&instance.instance), "view")
    }

    Ok(())
}

pub(crate) fn post_poll_event(
    _instance: &Instance,
) -> Result<Option<EventDataBuffer>, XrWrapError> {
    Ok(None)
}
