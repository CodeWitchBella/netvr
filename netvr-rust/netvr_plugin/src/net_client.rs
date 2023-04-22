use anyhow::{anyhow, Result};
use netvr_data::{
    bincode,
    net::{ConfigurationUp, LocalStateSnapshot},
    SendFrames,
};
use tokio::select;
use xr_layer::{log::LogTrace, sys, XrDebug};

use crate::{
    instance::{Instance, Session},
    overrides::with_layer,
};

/// Implements the netvr client state machine. Should be recalled if it exists
/// to reconnect to the server.
pub(crate) async fn run_net_client(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    LogTrace::str("connecting to netvr server...");
    let connection =
        netvr_client::connect(|text| LogTrace::string(format!("[conn] {}", text))).await?;
    LogTrace::string(format!(
        "Connected to netvr server: {:?}",
        connection.connection.remote_address()
    ));

    // alright, so we are connected let's send info about local devices...

    // read data: lock, clone, unlock

    // TODO
    let transmit_conf =
        run_transmit_configuration(connection.configuration_up, instance_handle, session_handle);
    let transmit_snap = run_transmit_snapshots(
        connection.connection.clone(),
        instance_handle,
        session_handle,
    );
    let receive = run_receive_snapshots(
        connection.connection.clone(),
        instance_handle,
        session_handle,
    );
    select! {
        value = transmit_conf => value,
        value = transmit_snap => value,
        value = receive => value,
    }
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
            .write(&ConfigurationUp::ConfigurationSnapshot(value))
            .await?;
        conf.changed().await?;
    }
}

async fn run_receive_snapshots(
    connection: quinn::Connection,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    loop {
        let datagram = connection.read_datagram().await?;
        let value: netvr_data::net::RemoteStateSnapshot = bincode::deserialize(&datagram)?;

        with_layer(instance_handle, |instance| {
            let session = instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to read session from instance"))?;
            let mut remote_state = session.remote_state.write().map_err(|err| {
                anyhow::anyhow!("Failed to acquire write lock on remote_state: {:?}", err)
            })?;
            if value.order > remote_state.order {
                *remote_state = value;
            }

            Ok(())
        })?;
    }
}

async fn run_transmit_snapshots(
    connection: quinn::Connection,
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<()> {
    loop {
        if let Some(value) = collect_state(instance_handle, session_handle)? {
            connection.send_datagram(bincode::serialize(&value)?.into())?;
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

/// Collects the state of the local devices. None means that the state could not
/// be collected and that it should be tried again. Errors are non-recoverable
/// and the connection loop should be ended.
fn collect_state(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<Option<LocalStateSnapshot>> {
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
fn collect_state_impl(instance: &Instance, session: &Session) -> Option<LocalStateSnapshot> {
    let time = instance.instance.now().ok()?;
    let location = session.space_view.locate(&session.space_stage, time).ok()?;

    Some(LocalStateSnapshot {
        controllers: vec![],
        views: vec![location.pose.into()],
    })
}
