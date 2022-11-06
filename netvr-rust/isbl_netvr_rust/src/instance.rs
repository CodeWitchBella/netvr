use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use xr_layer::{log::LogInfo, safe_openxr, sys};

use crate::xr_wrap::XrWrapError;

pub(crate) struct Session {
    pub(crate) session: safe_openxr::Session<safe_openxr::AnyGraphics>,
    pub(crate) view_configuration_type: safe_openxr::ViewConfigurationType,
    pub(crate) space_stage: RwLock<Option<safe_openxr::Space>>,
    pub(crate) time: sys::Time,
}

impl Session {
    pub(crate) fn new(
        session: safe_openxr::Session<safe_openxr::AnyGraphics>,
    ) -> Result<Self, XrWrapError> {
        Ok(Self {
            session,
            // this will be set later
            view_configuration_type: sys::ViewConfigurationType::PRIMARY_MONO,
            space_stage: RwLock::new(None),
            time: sys::Time::from_nanos(-1),
        })
    }

    pub(crate) fn read_space(
        &self,
    ) -> Result<RwLockReadGuard<Option<safe_openxr::Space>>, XrWrapError> {
        {
            let r = self
                .space_stage
                .read()
                .map_err(|_| sys::Result::ERROR_RUNTIME_FAILURE)?;
            if r.is_some() {
                return Ok(r);
            }
        };

        {
            let mut w = self
                .space_stage
                .write()
                .map_err(|_| sys::Result::ERROR_RUNTIME_FAILURE)?;
            if w.is_none() {
                w.replace(self.session.create_reference_space(
                    safe_openxr::ReferenceSpaceType::STAGE,
                    safe_openxr::Posef::IDENTITY,
                )?);
            };
        }

        Ok(self
            .space_stage
            .read()
            .map_err(|_| sys::Result::ERROR_RUNTIME_FAILURE)?)
    }
}

pub(crate) struct Instance {
    pub(crate) instance: safe_openxr::Instance,
    pub(crate) sessions: HashMap<sys::Session, Session>,
}

impl Instance {
    pub(crate) fn new(instance: safe_openxr::Instance) -> Self {
        Self {
            instance,
            sessions: HashMap::default(),
        }
    }

    pub(crate) fn fp(&self) -> &xr_layer::raw::Instance {
        self.instance.fp()
    }
}
