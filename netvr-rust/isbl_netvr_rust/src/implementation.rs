use std::{error::Error, ffi::CStr, os::raw::c_char};

use xr_layer::{
    log::{LogInfo, LogWarn},
    openxr, CreateAction, LayerImplementation, SyncActions, XrResult,
};

/*
fn parse_input_string<'a>(name_ptr: *const c_char) -> Option<&'a str> {
    match unsafe { CStr::from_ptr(name_ptr) }.to_str() {
        Ok(val) => Some(val),
        Err(error) => {
            LogWarn::string(format!(
                "Failed to parse string input as UTF8. Error: {}",
                error.source().unwrap(),
            ));
            None
        }
    }
}
 */

pub struct ImplementationInstance {}
impl LayerImplementation for ImplementationInstance {
    fn new(_lower: &openxr::Instance) -> Self {
        Self {}
    }

    fn sync_actions(&self, input: SyncActions) -> XrResult<()> {
        let result = input.sync();
        LogInfo::string(format!("xrSyncActions {:#?} -> {:?}", input, result));
        result
    }

    fn create_action(&self, input: CreateAction) -> XrResult<()> {
        for ptr in input.info() {
            if let Some(info) = ptr.read_action_create_info() {
                LogInfo::string(format!(
                    "info {:#?} {:#?}",
                    //parse_input_string(info.action_name),
                    info.action_name,
                    info.action_type
                ));
            }
        }
        let result = input.create_action();
        LogInfo::string(format!("xrCreateAction {:#?} -> {:?}", input, result));
        result
    }
}
