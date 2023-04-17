use anyhow::Result;
use xr_layer::log::LogInfo;

/// Implements the netvr client state machine. Should be recalled if it exists
/// to reconnect to the server.
pub(crate) async fn run_net_client() -> Result<()> {
    let connection = netvr_client::connect().await?;
    LogInfo::string(format!(
        "Connected to netvr server: {:?}",
        connection.connection.remote_address()
    ));

    // alright, so we are connected let's send info about local devices...
    // TODO

    // forever
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
