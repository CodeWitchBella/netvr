use std::collections::HashMap;

use netvr_data::net::{
    ActionType, RemoteAction, RemoteConfigurationSnapshot, RemoteInteractionProfile,
};
use xr_layer::sys;

#[derive(Default, Clone, Debug)]
pub(crate) struct Action {
    pub ty: ActionType,
    pub name: String,
    pub localized_name: String,
    pub binding: String,
    /// Maps subaction path to bound space. None for non-pose actions.
    pub(crate) spaces: Option<HashMap<sys::Path, sys::Space>>,
}

impl From<Action> for RemoteAction {
    fn from(val: Action) -> Self {
        RemoteAction {
            ty: val.ty,
            name: val.name,
            localized_name: val.localized_name,
            binding: val.binding,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct InteractionProfile {
    pub path: String,
    pub bindings: Vec<Action>,
    pub path_handle: sys::Path,
}

impl From<InteractionProfile> for RemoteInteractionProfile {
    fn from(val: InteractionProfile) -> Self {
        RemoteInteractionProfile {
            path: val.path,
            bindings: val.bindings.into_iter().map(|b| b.into()).collect(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct LocalConfigurationSnapshot {
    pub version: u32,
    pub interaction_profiles: Vec<InteractionProfile>,
    pub user_paths: Vec<(sys::Path, String)>,
    pub name: String,
}

impl From<LocalConfigurationSnapshot> for RemoteConfigurationSnapshot {
    fn from(val: LocalConfigurationSnapshot) -> Self {
        RemoteConfigurationSnapshot {
            version: val.version,
            interaction_profiles: val
                .interaction_profiles
                .into_iter()
                .map(|p| p.into())
                .collect(),
            user_paths: val.user_paths.iter().map(|(_, p)| p.clone()).collect(),
            name: val.name,
        }
    }
}
