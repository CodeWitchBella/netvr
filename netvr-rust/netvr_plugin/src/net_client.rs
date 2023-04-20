use anyhow::{anyhow, Result};
use netvr_data::{bincode, net::LocalStateSnapshot};
use tokio::select;
use xr_layer::{log::LogTrace, sys};

use crate::{instance::Session, overrides::with_layer};

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
    let transmit = run_transmit_snapshots(
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
        value = transmit => value,
        value = receive => value,
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
    Ok(with_layer(instance_handle, |instance| {
        Ok(collect_state_impl(
            instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to get session data"))?,
        ))
    })?)
}

/// Collects the state of the local devices. None means that the state could not
/// be collected.
fn collect_state_impl(session: &Session) -> Option<LocalStateSnapshot> {
    let r = session.read_space().ok()?;
    let space = if let Some(val) = &*r {
        val
    } else {
        return None;
    };
    let time = session.time;
    if time.as_nanos() < 0 {
        return None;
    }

    let (_info, views) = session
        .session
        .locate_views(session.view_configuration_type, time, space)
        .ok()?;

    Some(LocalStateSnapshot {
        controllers: vec![],
        views: views.into_iter().map(|v| v.pose.into()).collect(),
    })
}
