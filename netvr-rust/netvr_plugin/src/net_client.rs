use xr_layer::log::{LogError, LogInfo};

/// Implements the netvr client state machine. Should be recalled if it exists
/// to reconnect to the server.
pub(crate) async fn run_net_client() {
    if let Ok(connection) = netvr_client::connect().await {
        LogInfo::string(format!(
            "Connected to netvr server: {:?}",
            connection.connection.remote_address()
        ));
    } else {
        LogError::str("Failed to connect to netvr server");
    }
}
