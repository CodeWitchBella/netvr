use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::{atomic::AtomicBool, mpsc, Arc, Mutex, RwLock},
};

use anyhow::{anyhow, Result};
use netvr_data::{
    app,
    net::{self, ConfigurationSnapshotSet, RemoteStateSnapshotSet},
    Pose, RemoteSnapshot,
};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tracing::{span, Level, Span};
use xr_layer::{
    log::LogTrace,
    safe_openxr,
    sys::{self},
    XrDebug,
};

use crate::{local_configuration::LocalConfigurationSnapshot, xr_wrap::Trace};

/// This struct has 1-1 correspondence with each session the application creates
/// It is used to hold the underlying session from runtime and extra data
/// required by the netvr layer.
pub(crate) struct Session {
    pub(crate) session: safe_openxr::Session<safe_openxr::AnyGraphics>,
    pub(crate) view_configuration_type: safe_openxr::ViewConfigurationType,
    pub(crate) space_stage: safe_openxr::Space,
    pub(crate) application_stage_spaces: Arc<RwLock<HashSet<sys::Space>>>,
    pub(crate) space_view: safe_openxr::Space,
    pub(crate) space_server: Arc<RwLock<safe_openxr::Space>>,
    pub(crate) server_address: Arc<RwLock<Option<String>>>,
    pub(crate) predicted_display_time: sys::Time,
    pub(crate) token: CancellationToken,
    pub(crate) started_session: AtomicBool,

    /// Maps user paths (eg. /user/hand/left) to active interaction profile for
    /// it (eg. /interaction_profiles/khr/simple_controller).
    pub(crate) active_interaction_profiles: Arc<RwLock<HashMap<sys::Path, sys::Path>>>,
    pub(crate) local_configuration: watch::Sender<LocalConfigurationSnapshot>,

    /// This contains data that is received from the server and is made
    /// available to the application.
    pub(crate) remote_state: Arc<RwLock<RemoteStateSnapshotSet>>,
    pub(crate) remote_configuration: Arc<RwLock<ConfigurationSnapshotSet>>,

    pub(crate) remote_app_state: Arc<RwLock<app::Snapshot>>,
    pub(crate) local_app_overrides: Arc<RwLock<HashMap<usize, Pose>>>,
    pub(crate) grabbed: Arc<RwLock<HashSet<u32>>>,

    pub(crate) remote_merged: Arc<RwLock<RemoteSnapshot>>,
    _span: Span,
}

impl Session {
    /// Initializes the structure.
    pub(crate) fn new(
        session: safe_openxr::Session<safe_openxr::AnyGraphics>,
        trace: &Trace,
    ) -> Result<Self> {
        let stage = session.create_reference_space(
            safe_openxr::ReferenceSpaceType::STAGE,
            safe_openxr::Posef::IDENTITY,
        )?;
        let view = session.create_reference_space(
            safe_openxr::ReferenceSpaceType::VIEW,
            safe_openxr::Posef::IDENTITY,
        )?;
        let server = session.create_reference_space(
            safe_openxr::ReferenceSpaceType::STAGE,
            safe_openxr::Posef::IDENTITY,
        )?;
        Ok(Self {
            session,
            // this will be set later
            view_configuration_type: sys::ViewConfigurationType::PRIMARY_MONO,
            space_stage: stage,
            application_stage_spaces: Default::default(),
            space_view: view,
            space_server: Arc::new(RwLock::new(server)),
            server_address: Arc::new(RwLock::new(None)),
            predicted_display_time: sys::Time::from_nanos(-1),
            active_interaction_profiles: Arc::default(),
            local_configuration: watch::channel(Default::default()).0,
            token: CancellationToken::new(),
            started_session: AtomicBool::new(false),

            remote_state: Arc::default(),
            remote_configuration: Arc::default(),
            remote_merged: Arc::default(),

            remote_app_state: Arc::default(),
            local_app_overrides: Arc::default(),
            grabbed: Arc::default(),

            _span: trace.wrap(|| span!(Level::TRACE, "Instance")),
        })
    }

    pub(crate) fn update_merged(&self) -> Result<()> {
        let state = self
            .remote_state
            .read()
            .map_err(|err| anyhow!("{:?}", err))?;
        let config = self
            .remote_configuration
            .read()
            .map_err(|err| anyhow!("{:?}", err))?;
        let mut merged = self
            .remote_merged
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;

        merged
            .clients
            .retain(|client_id, _client| config.clients.contains_key(client_id));

        for (client_id, client_config) in &config.clients {
            let Some(state) = state.clients.get(client_id) else { continue; };
            if client_config.version == state.required_configuration {
                merged.clients.insert(
                    *client_id,
                    netvr_data::RemoteClientSnapshot {
                        configuration: client_config.clone(),
                        state: state.clone(),
                    },
                );
            }
        }

        for client_id in merged.clients.keys().cloned().collect::<Vec<_>>() {
            if !config.clients.contains_key(&client_id) {
                merged.clients.remove(&client_id);
            }
        }

        for (client_id, client) in &mut merged.clients {
            let old_config = client.configuration.clone();
            let old_state = client.state.clone();

            let new_config = config.clients[client_id].clone();
            let new_state = state.clients.get(client_id).cloned();

            let Some(new_state) = new_state else { continue; };

            #[allow(clippy::comparison_chain)]
            if old_config.version == new_config.version {
                if new_state.required_configuration == old_config.version {
                    client.state = new_state;
                }
            } else if new_config.version > old_config.version {
                if new_state.required_configuration == new_config.version {
                    client.configuration = new_config;
                    client.state = new_state;
                } else if old_state.required_configuration == new_config.version {
                    client.configuration = new_config;
                } else if new_state.required_configuration == old_config.version {
                    client.state = new_state;
                }
            }
        }
        Ok(())
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

/// Data stored relevant to this OpenXR object
#[derive(Debug, Clone)]
pub(crate) struct Action {
    pub(crate) handle: sys::Action,
    // key is interaction profile, value is the binding
    pub(crate) path: HashMap<sys::Path, sys::Path>,
    pub(crate) typ: net::ActionType,
    pub(crate) name: String,
    pub(crate) localized_name: String,
    pub(crate) subaction_paths: Vec<sys::Path>,
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
/// Data stored relevant to this OpenXR object
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
