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
