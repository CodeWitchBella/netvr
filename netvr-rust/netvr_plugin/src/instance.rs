use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{mpsc, Arc, Mutex, RwLock, RwLockReadGuard},
};

use tokio_util::sync::CancellationToken;
use tracing::{span, Level, Span};
use xr_layer::{log::LogInfo, safe_openxr, sys};

use crate::{
    data::Data,
    xr_wrap::{Trace, XrWrapError},
};

/// This struct has 1-1 correspondence with each session the application creates
/// It is used to hold the underlying session from runtime and extra data
/// required by the netvr layer.
pub(crate) struct Session {
    pub(crate) session: safe_openxr::Session<safe_openxr::AnyGraphics>,
    pub(crate) view_configuration_type: safe_openxr::ViewConfigurationType,
    pub(crate) space_stage: RwLock<Option<safe_openxr::Space>>,
    pub(crate) time: sys::Time,
    _span: Span,
}

impl Session {
    /// Initializes the structure.
    pub(crate) fn new(
        session: safe_openxr::Session<safe_openxr::AnyGraphics>,
        trace: &Trace,
    ) -> Result<Self, XrWrapError> {
        Ok(Self {
            session,
            // this will be set later
            view_configuration_type: sys::ViewConfigurationType::PRIMARY_MONO,
            space_stage: RwLock::new(None),
            time: sys::Time::from_nanos(-1),
            _span: trace.wrap(|| span!(Level::TRACE, "Instance")),
        })
    }

    /// Reads space_stage and if it is None, then it tries to initialize it.
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

impl Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session", &self.session.as_raw())
            .field("view_configuration_type", &self.view_configuration_type)
            .field("time", &self.time)
            .finish_non_exhaustive()
    }
}

pub(crate) struct ViewData {
    pub pose: sys::Posef,
    #[allow(dead_code)]
    pub fov: sys::Fovf,
}

/// This struct has 1-1 correspondence with each XrInstance the application
/// creates It is used to hold the underlying instance from runtime and extra
/// data required by the netvr layer.
pub(crate) struct Instance {
    pub(crate) instance: safe_openxr::Instance,
    pub(crate) tokio: tokio::runtime::Handle,
    pub(crate) token: CancellationToken,
    pub(crate) finished_rx: Mutex<mpsc::Receiver<()>>,
    pub(crate) sessions: HashMap<sys::Session, Session>,
    pub(crate) views: Mutex<Vec<ViewData>>,
    /// This contains data that available to both the OpenXR part and the netvr
    /// client part.
    pub(crate) data: Arc<Data>,
    _span: Span,
}

impl Instance {
    /// Initializes the structure.
    pub(crate) fn new(instance: safe_openxr::Instance) -> Self {
        let (tx, rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let token = CancellationToken::new();
            // We want to run netvr plugin on one thread so as not to waste
            // resources available to the application.
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()
                .unwrap();

            let _ = tx.send((rt.handle().clone(), token.clone()));
            // Run here until cancelled.
            rt.block_on(token.cancelled());
            rt.shutdown_background();
            let _ = finished_tx.send(());
        });
        let (tokio, token) = rx.recv().unwrap();
        LogInfo::str("you should see this message");
        Self {
            instance,

            tokio,
            token,
            finished_rx: Mutex::new(finished_rx),
            sessions: HashMap::default(),
            views: Mutex::new(vec![]),
            data: Arc::new(Data::new()),
            _span: span!(Level::TRACE, "Instance"),
        }
    }

    /// Convenience function serving as a shortcut.
    pub(crate) fn fp(&self) -> &xr_layer::raw::Instance {
        self.instance.fp()
    }
}

impl Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("instance", &self.instance.as_raw())
            .finish_non_exhaustive()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.token.cancel();
        self.finished_rx.lock().unwrap().recv().unwrap();
    }
}
