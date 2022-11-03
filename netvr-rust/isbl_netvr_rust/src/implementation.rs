use xr_layer::{log::LogInfo, sys, XrIterator};

use crate::instance::Instance;

pub(crate) fn post_sync_actions(instance: &Instance, infos: XrIterator) {
    for info in infos {
        LogInfo::string(format!("post_sync_actions {:?}", info.ty));
    }
}

pub(crate) fn tick(handle: sys::Instance, instance: &Instance) {
    LogInfo::string(format!("tick {:?}", handle));
}
