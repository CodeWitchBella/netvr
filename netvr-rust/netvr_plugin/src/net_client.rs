use anyhow::Result;
use netvr_data::{bincode, net::DatagramUp};
use xr_layer::{log::LogTrace, sys};

use crate::overrides::with_layer;

/// Implements the netvr client state machine. Should be recalled if it exists
/// to reconnect to the server.
pub(crate) async fn run_net_client(instance_handle: sys::Instance) -> Result<()> {
    LogTrace::str("connecting to netvr server...");
    let connection =
        netvr_client::connect(|text| LogTrace::string(format!("[conn] {}", text))).await?;
    LogTrace::string(format!(
        "Connected to netvr server: {:?}",
        connection.connection.remote_address()
    ));

    // alright, so we are connected let's send info about local devices...
    let data_ref = with_layer(instance_handle, |instance| Ok(instance.data.clone()))?;
    // read data: lock, clone, unlock

    // TODO

    // forever
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let data = { data_ref.lock().unwrap().clone() };
        connection
            .connection
            .send_datagram(bincode::serialize(&DatagramUp { state: data.state })?.into())?;
    }
}
