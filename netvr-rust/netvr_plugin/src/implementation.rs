use std::collections::hash_map::Entry::Occupied;

use anyhow::{anyhow, Result};
use netvr_data::{
    app, GrabInput, InitRemoteObjectsInput, InstanceAndSession, Nothing, ReadRemoteDevicesOutput,
    RemoteDevice, SetPoseInput, StartInput,
};
use tokio::select;
use tracing::info;
use xr_layer::log::{LogInfo, LogTrace};

use crate::{net_client::run_net_client, overrides::with_layer};

/// Starts the netvr client. Should be called after xrCreateInstance.
pub(crate) fn start(input: StartInput) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        info!("start {:?}", instance.instance.as_raw());

        let token = instance.token.clone();
        instance.tokio.spawn(async move {
            loop {
                select! {
                    _ = token.cancelled() => { break; }
                    res = run_net_client(input.instance, input.session, input.data_directory.clone()) => {
                        LogInfo::string(format!("net_client finished {:?}", res));
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        Ok(Nothing::default())
    })
}

pub(crate) fn read_remote_devices(
    input: InstanceAndSession,
) -> Result<netvr_data::ReadRemoteDevicesOutput> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let mut devices = ReadRemoteDevicesOutput::default();

        let remote_merged = session
            .remote_merged
            .read()
            .map_err(|err| anyhow!("{:?}", err))?;

        for (client_id, client) in remote_merged.clients.iter() {
            let mut i = 0;
            {
                let device = client.state.view.clone();
                i += 1;
                devices.devices.push(RemoteDevice {
                    id: client_id * 100 + i,
                    pos: device.position,
                    rot: device.orientation,
                    user_path: "/user/head".to_owned(),
                    interaction_profile: "generic_hmd".to_owned(),
                });
            }
            for device in client.state.controllers.iter() {
                let Some(interaction_profile) = client
                    .configuration
                    .interaction_profiles
                    .get(usize::from(device.interaction_profile) - 1) else { continue; };
                let Some(user_path) = client
                    .configuration
                    .user_paths
                    .get(usize::from(device.user_path) - 1)
                     else { continue; };
                i += 1;
                devices.devices.push(RemoteDevice {
                    id: client_id * 100 + i,
                    pos: device.pose.position.clone(),
                    rot: device.pose.orientation.clone(),
                    user_path: user_path.clone(),
                    interaction_profile: interaction_profile.path.clone(),
                });
            }
        }
        Ok(devices)
    })
}

pub(crate) fn read_remote_objects(input: InstanceAndSession) -> Result<app::Snapshot> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;

        let mut remote_app_state = {
            session
                .remote_app_state
                .read()
                .map_err(|err| anyhow!("{:?}", err))?
                .clone()
        };
        let local_app_overrides = session
            .local_app_overrides
            .read()
            .map_err(|err| anyhow!("{:?}", err))?;
        for (id, o) in &*local_app_overrides {
            remote_app_state.objects[*id] = o.clone();
        }

        Ok(remote_app_state)
    })
}

pub(crate) fn init_remote_objects(input: InitRemoteObjectsInput) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let mut lock = session
            .remote_app_state
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;
        LogTrace::string(format!("init_remote_objects {:?}", input.snapshot));
        if lock.objects.is_empty() {
            *lock = input.snapshot;
        }
        Ok(Nothing::default())
    })
}

pub(crate) fn grab(input: GrabInput) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let pose = {
            session
                .remote_app_state
                .read()
                .map_err(|err| anyhow!("{:?}", err))?
                .objects
                .get(usize::try_from(input.object_id)?)
                .ok_or(anyhow!("Object not found"))?
                .clone()
        };
        let mut grabbed = session
            .grabbed
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;
        LogTrace::string(format!("grab {:?}", input.object_id));
        grabbed.insert(input.object_id);
        let mut local_app_overrides = session
            .local_app_overrides
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;
        local_app_overrides.insert(usize::try_from(input.object_id)?, pose);
        Ok(Nothing::default())
    })
}

pub(crate) fn release(input: GrabInput) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let mut local_app_overrides = session
            .local_app_overrides
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;
        LogTrace::string(format!("release {:?}", input.object_id));
        local_app_overrides.remove(&usize::try_from(input.object_id)?);
        Ok(Nothing::default())
    })
}

pub(crate) fn object_set_pose(input: SetPoseInput) -> Result<Nothing> {
    with_layer(input.instance, |instance| {
        let session = instance
            .sessions
            .get(&input.session)
            .ok_or(anyhow!("Session not found"))?;
        let mut local_app_overrides = session
            .local_app_overrides
            .write()
            .map_err(|err| anyhow!("{:?}", err))?;
        LogTrace::string(format!(
            "object_set_pose {:?} {:?}",
            input.object_id, input.pose
        ));
        if let Occupied(mut e) = local_app_overrides.entry(usize::try_from(input.object_id)?) {
            e.insert(input.pose);
        }
        Ok(Nothing::default())
    })
}
