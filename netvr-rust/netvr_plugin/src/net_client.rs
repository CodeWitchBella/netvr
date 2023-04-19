use anyhow::{anyhow, Result};
use netvr_data::{bincode, net::LocalStateSnapshot};
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

    // forever
    loop {
        let value = collect_state(instance_handle, session_handle)?;
        connection
            .connection
            .send_datagram(bincode::serialize(&value)?.into())?;

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

fn collect_state(
    instance_handle: sys::Instance,
    session_handle: sys::Session,
) -> Result<LocalStateSnapshot> {
    with_layer(instance_handle, |instance| {
        Ok(collect_state_impl(
            instance
                .sessions
                .get(&session_handle)
                .ok_or(anyhow!("Failed to get session data"))?,
        ))
    })?
}

fn collect_state_impl(session: &Session) -> Result<LocalStateSnapshot> {
    let r = session.read_space()?;
    let space = if let Some(val) = &*r {
        val
    } else {
        return Err(anyhow::anyhow!("no space"));
    };
    let time = session.time;
    if time.as_nanos() < 0 {
        return Err(anyhow::anyhow!("invalid time"));
    }

    let (_info, views) =
        session
            .session
            .locate_views(session.view_configuration_type, time, space)?;

    Ok(LocalStateSnapshot {
        controllers: vec![],
        views: views.into_iter().map(|v| v.pose.into()).collect(),
    })
}
