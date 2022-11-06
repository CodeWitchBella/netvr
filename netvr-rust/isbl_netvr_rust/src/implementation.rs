use xr_layer::{log::LogInfo, safe_openxr, XrDebug, XrIterator};

use crate::{
    instance::{Instance, Session},
    xr_wrap::XrWrapError,
};

/// Called after sync_actions openxr call is performed, but before it returns to
/// application. This is when we should update structures read by overrides with
/// data we received from network.
///
/// This could also be opportunity to update data to be sent from local info.
///
pub(crate) fn post_sync_actions(_: &Instance, infos: XrIterator) {
    for info in infos {
        LogInfo::string(format!("post_sync_actions {:?}", info.ty));
    }
}

/// Should be periodically called from application. Sends data to network.
pub(crate) fn tick(instance: &Instance) -> Result<(), XrWrapError> {
    LogInfo::string(format!("tick {:?}", instance.instance.as_raw()));

    for session in instance.sessions.values() {
        if let Err(error) = tick_session(&instance.instance, session) {
            LogInfo::string(format!("  session_tick failed with: {:?}", error));
        }
    }
    Ok(())
}

/// Called for each session once per tick.
fn tick_session(instance: &safe_openxr::Instance, session: &Session) -> Result<(), XrWrapError> {
    LogInfo::string(format!("  session: {:?}", session.session.as_raw()));
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
    LogInfo::string(format!("  views: {:?}", info));
    for view in views {
        LogInfo::string(format!("  view: {:?}", view.pose.as_debug(instance)));
    }

    Ok(())
}
