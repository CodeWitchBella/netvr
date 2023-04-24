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
    pub extra: ActionExtra,
}

impl From<Action> for RemoteAction {
    fn from(action: Action) -> Self {
        Self {
            ty: action.ty,
            name: action.name,
            localized_name: action.localized_name,
            binding: action.binding,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct InteractionProfile {
    pub path: String,
    pub bindings: Vec<Action>,

    pub path_handle: u64,
}

impl From<InteractionProfile> for RemoteInteractionProfile {
    fn from(profile: InteractionProfile) -> Self {
        Self {
            path: profile.path,
            bindings: profile.bindings.into_iter().map(|b| b.into()).collect(),
            path_handle: profile.path_handle,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct LocalConfigurationSnapshot {
    pub version: u32,
    pub interaction_profiles: Vec<InteractionProfile>,
}

impl From<LocalConfigurationSnapshot> for RemoteConfigurationSnapshot {
    fn from(snapshot: LocalConfigurationSnapshot) -> Self {
        Self {
            version: snapshot.version,
            interaction_profiles: snapshot
                .interaction_profiles
                .into_iter()
                .map(|p| p.into())
                .collect(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct ActionExtra {
    pub(crate) action: sys::Action,
    /// Maps subaction path to bound space.
    pub(crate) spaces: Option<HashMap<sys::Path, sys::Space>>,
}
