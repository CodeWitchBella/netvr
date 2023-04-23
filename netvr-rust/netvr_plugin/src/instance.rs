use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{mpsc, Arc, Mutex, RwLock},
};

use anyhow::Result;
use netvr_data::{
    net::{self, ExtraMarker, LocalConfigurationSnapshot, RemoteStateSnapshot},
    serde::{Deserialize, Serialize},
};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tracing::{span, Level, Span};
use xr_layer::{log::LogTrace, safe_openxr, sys, XrDebug};

use crate::xr_wrap::Trace;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct ActionExtra {
    #[serde(with = "netvr_data::handle_serializer::action")]
    pub(crate) action: sys::Action,
}
impl ExtraMarker for ActionExtra {}

/// This struct has 1-1 correspondence with each session the application creates
/// It is used to hold the underlying session from runtime and extra data
/// required by the netvr layer.
pub(crate) struct Session {
    pub(crate) session: safe_openxr::Session<safe_openxr::AnyGraphics>,
    pub(crate) view_configuration_type: safe_openxr::ViewConfigurationType,
    pub(crate) space_stage: safe_openxr::Space,
    pub(crate) space_view: safe_openxr::Space,
    pub(crate) predicted_display_time: sys::Time,

    /// Maps user paths (eg. /user/hand/left) to active interaction profile for
    /// it (eg. /interaction_profiles/khr/simple_controller).
    pub(crate) active_interaction_profiles: Arc<RwLock<HashMap<sys::Path, sys::Path>>>,
    pub(crate) local_configuration: watch::Sender<LocalConfigurationSnapshot<ActionExtra>>,

    /// This contains data that is received from the server and is made
    /// available to the application.
    pub(crate) remote_state: Arc<RwLock<RemoteStateSnapshot>>,
    _span: Span,
}

impl Session {
    /// Initializes the structure.
    pub(crate) fn new(
        session: safe_openxr::Session<safe_openxr::AnyGraphics>,
        trace: &Trace,
    ) -> Result<Self> {
        // TODO: add server space
        let stage = session.create_reference_space(
            safe_openxr::ReferenceSpaceType::STAGE,
            safe_openxr::Posef::IDENTITY,
        )?;
        let view = session.create_reference_space(
            safe_openxr::ReferenceSpaceType::VIEW,
            safe_openxr::Posef::IDENTITY,
        )?;
        Ok(Self {
            session,
            // this will be set later
            view_configuration_type: sys::ViewConfigurationType::PRIMARY_MONO,
            space_stage: stage,
            space_view: view,
            predicted_display_time: sys::Time::from_nanos(-1),
            active_interaction_profiles: Arc::default(),
            local_configuration: watch::channel(LocalConfigurationSnapshot::default()).0,

            remote_state: Arc::default(),
            _span: trace.wrap(|| span!(Level::TRACE, "Instance")),
        })
    }
}

impl Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session", &self.session.as_raw())
            .field("view_configuration_type", &self.view_configuration_type)
            .field("time", &self.predicted_display_time)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Action {
    pub(crate) handle: sys::Action,
    // key is interaction profile, value is the binding
    pub(crate) path: HashMap<sys::Path, sys::Path>,
    pub(crate) typ: net::ActionType,
    pub(crate) name: String,
    pub(crate) localized_name: String,
}

impl XrDebug for Action {
    fn xr_fmt(
        &self,
        f: &mut std::fmt::Formatter,
        instance: &safe_openxr::Instance,
    ) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("handle", &self.handle)
            .field("name", &self.name)
            .field("path", &self.path.as_debug(instance))
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ActionSet {
    pub(crate) actions: Vec<Action>,
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

    pub(crate) action_sets: RwLock<HashMap<sys::ActionSet, ActionSet>>,

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
            LogTrace::str("token cancelled");
            rt.shutdown_background();
            let _ = finished_tx.send(());
            LogTrace::str("fully finished");
        });
        let (tokio, token) = rx.recv().unwrap();
        LogTrace::str("tokio handle received");
        Self {
            instance,

            tokio,
            token,
            finished_rx: Mutex::new(finished_rx),
            sessions: HashMap::default(),
            action_sets: RwLock::default(),

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
