use binary_layout::prelude::*;
use tracing::{info, instrument};
use xr_layer::{log::LogError, EventDataBuffer, XrDebug};

use crate::{
    instance::{Instance, Session},
    xr_wrap::XrWrapError,
};

/// Should be periodically called from application. Sends data to network.
pub(crate) fn tick(instance: &Instance) -> Result<(), XrWrapError> {
    info!("tick {:?}", instance.instance.as_raw());

    for session in instance.sessions.values() {
        if let Err(error) = tick_session(instance, session) {
            LogError::string(format!("session_tick failed with: {:?}", error));
        }
    }
    Ok(())
}

define_layout!(device, LittleEndian, {
    id: u32,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
});

pub(crate) const DEVICE_SIZE_CONST: usize = device::SIZE.unwrap();

pub(crate) fn read_remote_device_data(
    instance: &Instance,
    devices: &mut Vec<device::View<&mut [u8; DEVICE_SIZE_CONST]>>,
) -> Result<(), XrWrapError> {
    let vec = instance.views.lock().map_err(|err| err.to_string())?;
    for i in 0..devices.len() {
        let device = &mut devices[i];
        if let Some(v) = vec.get(i) {
            device.id_mut().write((i + 1).try_into().unwrap());
            device.x_mut().write(v.pose.position.x);
            device.y_mut().write(v.pose.position.y);
            device.z_mut().write(v.pose.position.z);
            device.qx_mut().write(v.pose.orientation.x);
            device.qy_mut().write(v.pose.orientation.y);
            device.qz_mut().write(v.pose.orientation.z);
            device.qw_mut().write(v.pose.orientation.w);
        } else {
            device.id_mut().write(0);
            device.x_mut().write(0.0);
            device.y_mut().write(1.0);
            device.z_mut().write(0.0);
            device.qx_mut().write(0.0);
            device.qy_mut().write(0.0);
            device.qz_mut().write(0.0);
            device.qw_mut().write(1.0);
        }
    }
    for device in devices {}
    Ok(())
}

pub(crate) fn read_remote_device_data_count(instance: &Instance) -> Result<usize, XrWrapError> {
    Ok(instance.views.lock().map_err(|err| err.to_string())?.len())
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
