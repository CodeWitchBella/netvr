use xr_layer::{log::LogInfo, safe_openxr, sys, ViewLocateInfo, XrIterator};

use crate::instance::Instance;

pub(crate) fn post_sync_actions(instance: &Instance, infos: XrIterator) {
    for info in infos {
        LogInfo::string(format!("post_sync_actions {:?}", info.ty));
    }
}

pub(crate) fn tick(instance: &safe_openxr::Instance) {
    LogInfo::string(format!("tick {:?}", instance.as_raw()));

    /*
    for session_handle in &instance.sessions {
        let session = safe_openxr::Session::from_raw();
        LogInfo::string(format!("  session: {:?}", session));
        let view_locate_info = sys::ViewLocateInfo {
            ty: todo!(),
            next: todo!(),
            view_configuration_type: todo!(),
            display_time: todo!(),
            space: todo!(),
        };
        let mut view_state = sys::ViewState {
            ty: todo!(),
            next: std::ptr::null().as_mut(),
            view_state_flags: todo!(),
        };
        let mut view = sys::View {
            ty: sys::View::TYPE,
            next: todo!(),
            pose: todo!(),
            fov: todo!(),
        };
        (instance.pfn.locate_views)(*session, view_locate_info, view_state, 1, pfn::NULL, view);
    } */
}
