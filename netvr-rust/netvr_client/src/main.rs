use std::time::Duration;

use netvr_client::connect;
use quinn::VarInt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello there! I'm looking for NetVR devices...");
    let conn = connect(|text| println!("[discovery] {}", text)).await?;
    let connection = conn.connection;
    println!("  remote_address: {:?}", connection.remote_address());
    println!("  local: {:?}", connection.local_ip());

    tokio::time::sleep(Duration::from_millis(100)).await;
    connection.close(VarInt::from_u32(0), b"Bye");

    connection.closed().await;
    println!("Connection closed");

    Ok(())
}
