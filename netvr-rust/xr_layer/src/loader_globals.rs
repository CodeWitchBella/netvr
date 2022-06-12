use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use crate::log::{LogError, LogWarn};

#[derive(Clone)]
pub struct ImplementationInstancePtr(pub *mut ::std::ptr::NonNull<::std::os::raw::c_void>);
unsafe impl Send for ImplementationInstancePtr {}
unsafe impl Sync for ImplementationInstancePtr {}

pub(crate) struct LayerInstance {
    // TODO: maybe use Box<dyn ImplementationTrait> instead?
    pub implementation: Option<ImplementationInstancePtr>,
    pub instance: openxr::Instance,
}

impl Drop for LayerInstance {
    fn drop(&mut self) {
        if self.implementation.is_some() {
            LogWarn::str("LayerInstance was not properly disposed of in Loader");
        }
    }
}

pub(crate) struct GlobalMaps {
    instances: RwLock<HashMap<u64, LayerInstance>>,
    sessions: RwLock<HashMap<u64, openxr_sys::Instance>>,
    action_sets: RwLock<HashMap<u64, openxr_sys::Instance>>,
}

impl GlobalMaps {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            action_sets: RwLock::new(HashMap::new()),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// READING LayerInstance directly
///////////////////////////////////////////////////////////////////////////////

impl GlobalMaps {
    pub fn get_instance_direct<'a>(
        &'a self,
        caller: &'static str,
        handle: openxr_sys::Instance,
    ) -> Result<InstanceDirectReadLock<'a>, openxr_sys::Result> {
        let guard = match self.instances.read() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?;
        Ok(InstanceDirectReadLock {
            guard,
            handle,
            caller,
        })
    }
}

pub(crate) struct InstanceDirectReadLock<'a> {
    pub guard: RwLockReadGuard<'a, HashMap<u64, LayerInstance>>,
    pub handle: openxr_sys::Instance,
    pub caller: &'static str,
}

impl<'a> InstanceDirectReadLock<'a> {
    pub fn read(&'a self) -> Result<&'a LayerInstance, openxr_sys::Result> {
        let handle = self.handle.into_raw();
        if let Some(instance) = (*self.guard).get(&handle) {
            Ok(instance)
        } else {
            LogError::string(format!(
                "{}: Can't find instance with handle {:?}. Maybe it was destroyed already?",
                self.caller, self.handle,
            ));
            Err(openxr_sys::Result::ERROR_HANDLE_INVALID)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// WRITING LayerInstance directly
///////////////////////////////////////////////////////////////////////////////

impl GlobalMaps {
    pub fn insert_instance(
        &self,
        handle: openxr_sys::Instance,
        layer_instance: LayerInstance,
    ) -> Result<(), String> {
        let mut guard = match self.instances.write() {
            Ok(v) => Ok(v),
            Err(err) => Err(format!(
                "insert_instance: Failed to acquire write lock on instances. {:}",
                err
            )),
        }?;

        let handle_raw = handle.into_raw();
        if (*guard).contains_key(&handle_raw) {
            LogWarn::string(format!(
                "insert_instance: Layer instance with handle {:?} already exists. This is probably a bug.",
                handle_raw,
            ));
        }
        (*guard).insert(handle_raw, layer_instance);

        Ok(())
    }

    pub fn remove_instance(&self, handle: openxr_sys::Instance) -> Result<LayerInstance, String> {
        let handle_raw = handle.into_raw();
        let mut guard = match self.instances.write() {
            Ok(v) => Ok(v),
            Err(err) => Err(format!(
                "remove_instance: Failed to acquire write lock on instances. {:}",
                err
            )),
        }?;

        let clear = |m: &RwLock<HashMap<u64, openxr_sys::Instance>>| {
            match m.write() {
                Ok(mut map) => map.retain(|_, v| v.into_raw() != handle_raw),
                Err(err) => LogWarn::string(format!(
                    "remove_instance: Failed to remove instance references with error {:?}",
                    err
                )),
            };
        };

        clear(&self.sessions);
        clear(&self.action_sets);

        match (*guard).remove(&handle_raw) {
            Some(instance) => Ok(instance),
            None => Err(format!(
                "remove_instance: Instance with handle {:} not found",
                handle_raw
            )),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// READING LayerInstance INdirectly
///////////////////////////////////////////////////////////////////////////////

impl GlobalMaps {
    pub fn get_instance<'a, Handle>(
        &'a self,
        caller: &'static str,
        handle: Handle,
    ) -> Result<InstanceReadLock<'a, Handle>, openxr_sys::Result>
    where
        Handle: GlobalMapsReadInstanceHandle + std::fmt::Debug,
    {
        let guard_instances = match self.instances.read() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?;
        let guard_map = match Handle::get_mapping(self).read() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?;
        Ok(InstanceReadLock {
            guard_instances,
            guard_map,
            handle,
            caller,
        })
    }
}

pub(crate) trait GlobalMapsReadInstanceHandle {
    fn get_mapping(maps: &GlobalMaps) -> &RwLock<HashMap<u64, openxr_sys::Instance>>;
    fn read_raw(&self) -> u64;
}

impl GlobalMapsReadInstanceHandle for openxr_sys::Session {
    fn get_mapping(maps: &GlobalMaps) -> &RwLock<HashMap<u64, openxr_sys::Instance>> {
        &maps.sessions
    }

    fn read_raw(&self) -> u64 {
        openxr_sys::Session::into_raw(*self)
    }
}

pub(crate) struct InstanceReadLock<'a, Handle>
where
    Handle: GlobalMapsReadInstanceHandle + std::fmt::Debug,
{
    pub guard_instances: RwLockReadGuard<'a, HashMap<u64, LayerInstance>>,
    pub guard_map: RwLockReadGuard<'a, HashMap<u64, openxr_sys::Instance>>,
    pub handle: Handle,
    pub caller: &'static str,
}

impl<'a, Handle> InstanceReadLock<'a, Handle>
where
    Handle: GlobalMapsReadInstanceHandle + std::fmt::Debug,
{
    pub fn read(&'a self) -> Result<&'a LayerInstance, openxr_sys::Result> {
        let handle_raw = self.handle.read_raw();
        if let Some(instance_handle) = (*self.guard_map).get(&handle_raw) {
            let instance_handle_raw = instance_handle.into_raw();
            match (*self.guard_instances).get(&instance_handle_raw) {
                Some(v) => Ok(v),
                None => {
                    LogError::string(format!(
                        "{}: Can't find instance for object with handle {:?}. This is probably a bug in instance cleanup.",
                        self.caller, self.handle,
                    ));
                    Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
                }
            }
        } else {
            LogError::string(format!(
                "{}: Can't find object with handle {:?}. Maybe it was destroyed already?",
                self.caller, self.handle,
            ));
            Err(openxr_sys::Result::ERROR_HANDLE_INVALID)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// WRITING LayerInstance indirection reference
///////////////////////////////////////////////////////////////////////////////

impl GlobalMaps {
    pub fn insert_instance_reference<Handle>(
        &self,
        caller: &'static str,
        handle: Handle,
        instance_handle: openxr_sys::Instance,
    ) -> Result<(), openxr_sys::Result>
    where
        Handle: GlobalMapsReadInstanceHandle + std::fmt::Debug,
    {
        let mut guard_map = match Handle::get_mapping(self).write() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?;
        (*guard_map).insert(handle.read_raw(), instance_handle);
        Ok(())
    }

    pub fn remove_instance_reference<'a, Handle>(
        &'a self,
        caller: &'static str,
        handle: Handle,
    ) -> Result<InstanceDirectReadLock<'a>, openxr_sys::Result>
    where
        Handle: GlobalMapsReadInstanceHandle + std::fmt::Debug,
    {
        let mut guard_map = match Handle::get_mapping(self).write() {
            Ok(v) => Ok(v),
            Err(err) => {
                LogError::string(format!(
                    "{}: Failed to acquire read lock on global instances. {:}",
                    caller, err
                ));
                Err(openxr_sys::Result::ERROR_RUNTIME_FAILURE)
            }
        }?;
        let handle_raw = handle.read_raw();
        match (*guard_map).remove(&handle_raw) {
            Some(instance_handle) => self.get_instance_direct(caller, instance_handle),
            None => Err(openxr_sys::Result::ERROR_HANDLE_INVALID),
        }
    }
}
