use std::{collections::HashSet, ptr, time::Duration};

use anyhow::{anyhow, Result};
use netvr_data::{
    app::{AppDatagramUp::SetPose, AppDown, AppUp},
    bincode,
    net::{
        self, ActionType, BaseSpace, CalibrationConfiguration, CalibrationSample, ConfigurationUp,
        DatagramDown, DatagramUp, StateSnapshot,
    },
    RecvFrames, SendFrames,
};
use tokio::{select, spawn, sync::mpsc, time};
use xr_layer::{
    log::{LogError, LogTrace},
    safe_openxr::{self},
    sys::{self, Space, Time},
};

use crate::{
    config::Config,
    instance::{Instance, Session},
    overrides::with_layer,
    xr_wrap::ResultConvertible,
};

macro_rules! map_err {
    ($text:expr) => {
        |err| anyhow::anyhow!(concat!($text, ": {:?}"), err)
    };
}

#[derive(Debug)]
enum CalibrationTrigger {
    Start(String, CalibrationConfiguration, BaseSpace),
    RequestSample(String, BaseSpace),
    Stop,
}

/// Implements the netvr client state machine. Should be recalled if it exists
/// to reconnect to the server.
pub(crate) async fn run_net_client(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    data_directory: String,
) -> Result<()> {
    LogTrace::str("connecting to netvr server...");
    let config = Config::load(data_directory).await;
    set_local_configuration_name(instance_handle, session_handle, config.name.clone())?;

    let connection =
        netvr_client::connect(|text| LogTrace::string(format!("[conn] {}", text))).await?;
    LogTrace::string(format!(
        "Connected to netvr server: {:?}",
        connection.connection.remote_address()
    ));
    let calibration_trigger = tokio::sync::mpsc::channel(1);

    // alright, so we are connected let's send info about local devices...

    let transmit_conf =
        run_transmit_configuration(connection.configuration_up, instance_handle, session_handle);
    let transmit_snap = run_transmit_snapshots(
        connection.connection.clone(),
        instance_handle,
        session_handle,
    );
    let receive = run_receive_datagrams(
        connection.connection.clone(),
        instance_handle,
        session_handle,
    );
    let recv_conf = run_receive_configuration(
        connection.configuration_down,
        instance_handle,
        session_handle,
        calibration_trigger.0,
        config,
    );
    let calibration_sender = spawn(run_calibration_sender(
        instance_handle,
        session_handle,
        connection.calibration_up,
        calibration_trigger.1,
    ));
    let send_app = run_send_app(
        instance_handle,
        session_handle,
        connection.app_up_stream,
        connection.connection.clone(),
    );
    let recv_app = run_recv_app(instance_handle, session_handle, connection.app_down_stream);
    select! {
        value = transmit_conf => value,
        value = transmit_snap => value,
        value = receive => value,
        value = recv_conf => value,
        value = calibration_sender => value?,
        value = send_app => value,
        value = recv_app => value,
    }
}

async fn run_recv_app(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    mut app_down: RecvFrames<AppDown>,
) -> Result<()> {
    loop {
        let message = app_down.read().await?;
        match message {
            AppDown::Release(object_id) => {
                with_layer(instance_handle, |instance| {
                    let session = instance
                        .sessions
                        .get(&session_handle)
                        .ok_or(anyhow!("Failed to read session from instance"))?;
                    let mut local_app_overrides = session
                        .local_app_overrides
                        .write()
                        .map_err(|err| anyhow!("Failed to read grabbed: {:?}", err))?;
                    local_app_overrides.remove(&usize::try_from(object_id)?);
                    Ok(())
                })?;
            }
        }
    }
}

async fn run_send_app(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    mut app_up: SendFrames<AppUp>,
    connection: quinn::Connection,
) -> Result<()> {
    let snap = with_layer(instance_handle, |instance| {
        let session = instance
            .sessions
            .get(&session_handle)
            .ok_or(anyhow!("Failed to read session from instance"))?;
        let lock = session
            .remote_app_state
            .read()
            .map_err(|err| anyhow!("Failed to read remote app state: {:?}", err))?;

        Ok(lock.clone())
    })?;
    app_up.write(&AppUp::Init(snap)).await?;

    let mut interval = tokio::time::interval(Duration::from_millis(20));
    loop {
        interval.tick().await;
        let (overrides, grabbed) = with_layer(instance_handle, |instance| {
            let session = instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to read session from instance"))?;
            let overrides = {
                session
                    .local_app_overrides
                    .read()
                    .map_err(|err| anyhow!("Failed to read remote app state: {:?}", err))?
                    .clone()
            };
            let grabbed = {
                let mut lock = session
                    .grabbed
                    .write()
                    .map_err(|err| anyhow!("Failed to read grabbed: {:?}", err))?;
                let grabbed = lock.clone();
                *lock = HashSet::new();
                grabbed
            };

            Ok((overrides, grabbed))
        })?;
        for g in grabbed {
            app_up.write(&AppUp::Grab(g)).await?;
        }

        for (id, pose) in &overrides {
            let bytes = bincode::serialize(&DatagramUp::App(SetPose(*id, pose.clone())))?;
            connection.send_datagram(bytes.into())?;
        }
    }
}

fn set_local_configuration_name(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    name: String,
) -> Result<()> {
    with_layer(instance_handle, |instance| {
        instance
            .sessions
            .get(&session_handle)
            .ok_or(anyhow!("Missing session"))?
            .local_configuration
            .send_modify(|conf| {
                conf.name = name;
            });
        Ok(())
    })
}

async fn run_transmit_configuration(
    mut connection: SendFrames<ConfigurationUp>,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    let mut conf = with_layer(instance_handle, |instance| {
        Ok(instance
            .sessions
            .get(&session_handle)
            .ok_or(anyhow!("Missing session"))?
            .local_configuration
            .subscribe())
    })?;
    loop {
        let value = conf.borrow().clone();

        connection
            .write(&ConfigurationUp::ConfigurationSnapshot(value.into()))
            .await?;
        conf.changed().await?;
    }
}

async fn run_receive_datagrams(
    connection: quinn::Connection,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    loop {
        let datagram = connection.read_datagram().await?;
        let value: netvr_data::net::DatagramDown = bincode::deserialize(&datagram)?;

        with_layer(instance_handle, |instance| {
            let session = instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to read session from instance"))?;

            match value {
                DatagramDown::App(value) => {
                    let mut state = session
                        .remote_app_state
                        .write()
                        .map_err(map_err!("Failed to acquire write lock on remote_app_state"))?;
                    if !value.objects.is_empty() {
                        *state = value;
                    }
                    Ok(())
                }
                DatagramDown::State(value) => {
                    {
                        let mut remote_state = session
                            .remote_state
                            .write()
                            .map_err(map_err!("Failed to acquire write lock on remote_state"))?;
                        if value.order > remote_state.order {
                            remote_state.clone_from(&value);
                        }
                    }
                    session.update_merged()?;

                    Ok(())
                }
            }
        })?;
    }
}

async fn run_calibration_sender(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    mut connection: netvr_data::SendFrames<netvr_data::net::CalibrationSample>,
    mut calibration_trigger: mpsc::Receiver<CalibrationTrigger>,
) -> Result<()> {
    loop {
        let Some(trigger) = calibration_trigger.recv().await else { break; };
        if let CalibrationTrigger::RequestSample(subaction_path, base_space) = trigger {
            let sample_time = now(instance_handle)?;
            connection
                .write(&collect_calibration_sample(
                    instance_handle,
                    session_handle,
                    subaction_path.as_str(),
                    sample_time,
                    None,
                    base_space,
                )?)
                .await?;
            continue;
        }

        let CalibrationTrigger::Start(subaction_path, conf, base_space) = trigger else { continue; };
        let duration = std::time::Duration::from_nanos(u64::try_from(conf.sample_interval_nanos)?);
        let duration_nanos = i64::try_from(duration.as_nanos())?;
        let mut interval = time::interval(duration);
        let mut sample_time = now(instance_handle)?;
        let mut prev_time = None;

        loop {
            connection
                .write(&collect_calibration_sample(
                    instance_handle,
                    session_handle,
                    subaction_path.as_str(),
                    sample_time,
                    prev_time,
                    base_space,
                )?)
                .await?;
            prev_time = Some(sample_time);
            sample_time = Time::from_nanos(sample_time.as_nanos() + duration_nanos);

            select! {
                trigger = calibration_trigger.recv() => match trigger {
                    Some(trigger) => match trigger {
                        CalibrationTrigger::Stop => break,
                        CalibrationTrigger::Start(..) => {
                            LogError::str("Calibration trigger received while already calibrating");
                        },
                        CalibrationTrigger::RequestSample(path, base_space) => {
                            LogError::string(format!("RequestSample({:?}, {:?}) received while already calibrating", path, base_space));
                        }
                    },
                    None => break,
                },
                _ = interval.tick() => {},
            }
        }
    }
    Ok(())
}

fn now(instance_handle: sys::Instance) -> Result<Time> {
    with_layer(instance_handle, |instance| Ok(instance.instance.now()?))
}

fn collect_calibration_sample(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    subaction_path: &str,
    time: Time,
    prev_time: Option<Time>,
    base_space: BaseSpace,
) -> Result<CalibrationSample> {
    with_layer(instance_handle, |instance| {
        let (action_space, base_space) =
            get_action_space(instance, session_handle, subaction_path, base_space)?;

        let now = instance.instance.now()?;
        let location = locate_space(
            &instance.instance,
            action_space.to_owned(),
            base_space,
            time,
        )?;
        let prev_location = prev_time
            .map(|prev_time| {
                locate_space(
                    &instance.instance,
                    action_space.to_owned(),
                    base_space,
                    prev_time,
                )
            })
            .transpose()?;

        Ok(CalibrationSample {
            flags: location.location_flags.into_raw(),
            pose: location.pose.into(),
            prev_flags: prev_location.map(|l| l.location_flags.into_raw()),
            prev_pose: prev_location.map(|l| l.pose.into()),
            nanos: time.as_nanos(),
            now_nanos: now.as_nanos(),
        })
    })
}

fn get_action_space(
    instance: &Instance,
    session_handle: sys::Session,
    subaction_path_str: &str,
    base_space: BaseSpace,
) -> Result<(Space, Space)> {
    let session = instance
        .sessions
        .get(&session_handle)
        .ok_or(anyhow!("Failed to read session from instance"))?;

    let base_space = match base_space {
        BaseSpace::Stage => session.space_stage.as_raw(),
        BaseSpace::Server => session
            .space_server
            .read()
            .map_err(|err| {
                anyhow!(format!(
                    "Failed to acquire read lock on space_server: {:?}",
                    err
                ))
            })?
            .as_raw(),
    };

    if subaction_path_str == "/user/head" {
        return Ok((session.space_view.as_raw(), base_space));
    }

    let active_profiles = session
        .active_interaction_profiles
        .read()
        .map_err(map_err!(
            "Failed to acquire read lock on active_interaction_profiles"
        ))?;
    let subaction_path = instance.instance.string_to_path(subaction_path_str)?;
    let conf = session.local_configuration.borrow();
    let (_, profile) = active_profiles
        .iter()
        .find(|(user_path, _)| **user_path == subaction_path)
        .ok_or(anyhow!(
            "Couldn't get active_profile with path: {:?}",
            subaction_path_str
        ))?;
    let interaction_profile = conf
        .interaction_profiles
        .iter()
        .find(|interaction_profile| interaction_profile.path_handle == *profile)
        .ok_or(anyhow!("Failed to get interaction_profile"))?;

    for binding in &interaction_profile.bindings {
        if let ActionType::Pose = binding.ty {
            let Some(spaces) = &binding.spaces else {continue;};
            let Some(space_action) = spaces.get(&subaction_path) else {continue;};
            return Ok((*space_action, base_space));
        }
    }
    Err(anyhow!("Failed to find pose binding"))
}

async fn run_receive_configuration(
    mut connection: netvr_data::RecvFrames<netvr_data::net::ConfigurationDown>,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
    calibration_trigger: mpsc::Sender<CalibrationTrigger>,
    mut config: Config,
) -> Result<()> {
    LogTrace::str("Waiting for configuration...");
    loop {
        let conf = connection.read().await?;

        LogTrace::string(format!("Received configuration: {:?}", conf));

        match conf {
            net::ConfigurationDown::Snapshot(snap) => with_layer(instance_handle, |instance| {
                let session = instance
                    .sessions
                    .get(&session_handle)
                    .ok_or(anyhow!("Failed to read session from instance"))?;
                {
                    let mut remote_configuration = session
                        .remote_configuration
                        .write()
                        .map_err(map_err!("Failed to acquire write lock on remote_state"))?;
                    remote_configuration.clone_from(&snap);
                }
                session.update_merged()?;

                Ok(())
            })?,
            net::ConfigurationDown::SetServerSpacePose(pose) => {
                with_layer(instance_handle, |instance| {
                    let session = instance
                        .sessions
                        .get(&session_handle)
                        .ok_or(anyhow!("Failed to read session from instance"))?;
                    {
                        let posef: sys::Posef = pose.into();
                        let space_server = session.session.create_reference_space(
                            safe_openxr::ReferenceSpaceType::STAGE,
                            posef,
                        )?;
                        let mut lock = session
                            .space_server
                            .write()
                            .map_err(map_err!("Failed to acquire write lock on space_server"))?;
                        *lock = space_server;
                        LogTrace::string(format!("Updated space_server: {:?}", posef));
                    }
                    session.update_merged()?;

                    Ok(())
                })?
            }
            net::ConfigurationDown::RequestSample(subaction_path, base_space) => {
                calibration_trigger
                    .send(CalibrationTrigger::RequestSample(
                        subaction_path,
                        base_space,
                    ))
                    .await?;
            }
            net::ConfigurationDown::TriggerCalibration(value, conf, base_space) => {
                calibration_trigger
                    .send(CalibrationTrigger::Start(value, conf, base_space))
                    .await?;
            }
            net::ConfigurationDown::StopCalibration => {
                calibration_trigger.send(CalibrationTrigger::Stop).await?;
            }
            net::ConfigurationDown::ChangeName(value) => {
                config.name = value;
                set_local_configuration_name(instance_handle, session_handle, config.name.clone())?;
                config.write().await;
            }
        };
    }
}

async fn run_transmit_snapshots(
    connection: quinn::Connection,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    loop {
        if let Some(value) = collect_state(instance_handle, session_handle)? {
            connection.send_datagram(bincode::serialize(&DatagramUp::State(value))?.into())?;
        }

        time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

/// Collects the state of the local devices. None means that the state could not
/// be collected and that it should be tried again. Errors are non-recoverable
/// and the connection loop should be ended.
fn collect_state(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<Option<StateSnapshot>> {
    with_layer(instance_handle, |instance| {
        Ok(collect_state_impl(
            instance,
            instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to get session data"))?,
        ))
    })
}

/// Collects the state of the local devices. None means that the state could not
/// be collected.
fn collect_state_impl(instance: &Instance, session: &Session) -> Option<StateSnapshot> {
    let time = instance.instance.now().ok()?;
    let space_server = session.space_server.read().ok()?;
    let view_location = session.space_view.locate(&space_server, time).ok()?;
    let active_profiles = session.active_interaction_profiles.read().ok()?;
    let conf = session.local_configuration.borrow();
    // TODO: collect full state
    let controllers = active_profiles
        .iter()
        .filter_map(|(user_path, profile)| {
            let (interaction_profile_index, interaction_profile) =
                conf.interaction_profiles
                    .iter().enumerate()
                    .find(|(_, interaction_profile)| {
                        interaction_profile.path_handle == *profile
                    })?;
            for binding in &interaction_profile.bindings {
                if let ActionType::Pose = binding.ty {
                    let Some(spaces) = &binding.spaces else {continue;};
                    let Some(space_action) = spaces.get(user_path) else {continue;};
                    let Ok(location) = locate_space(&instance.instance, space_action.to_owned(), space_server.as_raw(), time) else {continue;};
                    let Ok(interaction_profile_index) = u8::try_from(interaction_profile_index+1) else {continue;};
                    let Some(user_path_index) = conf.user_paths.iter().position(|p| p.0 == *user_path) else {continue;};
                    let Ok(user_path_index) = u8::try_from(user_path_index+1) else {continue;};
                    return Some(net::Controller {
                        pose: location.pose.into(),
                         interaction_profile: interaction_profile_index,
                        user_path: user_path_index
                    });
                }
            }
            None
        })
        .collect();

    Some(StateSnapshot {
        controllers,
        view: view_location.pose.into(),
        required_configuration: conf.version,
    })
}

fn locate_space(
    instance: &safe_openxr::Instance,
    space: sys::Space,
    base: sys::Space,
    time: sys::Time,
) -> Result<safe_openxr::SpaceLocation> {
    unsafe {
        let mut x = sys::SpaceLocation::out(ptr::null_mut());
        (instance.fp().locate_space)(space, base, time, x.as_mut_ptr()).into_result()?;
        let ptr = x.as_ptr();
        let flags = *ptr::addr_of!((*ptr).location_flags);
        Ok(safe_openxr::SpaceLocation {
            location_flags: flags,
            pose: sys::Posef {
                orientation: flags
                    .contains(sys::SpaceLocationFlags::ORIENTATION_VALID)
                    .then(|| *ptr::addr_of!((*ptr).pose.orientation))
                    .unwrap_or_default(),
                position: flags
                    .contains(sys::SpaceLocationFlags::POSITION_VALID)
                    .then(|| *ptr::addr_of!((*ptr).pose.position))
                    .unwrap_or_default(),
            },
        })
    }
}
