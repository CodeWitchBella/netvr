use netvr_client::connect;
use quinn::VarInt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello there! I'm looking for NetVR devices...");
    let (_endpoint, connection) = connect().await?;
    println!("  remote_address: {:?}", connection.remote_address());
    println!("  local: {:?}", connection.local_ip());
    println!("Opening heartbeat reciever...");
    let heartbeat = connection.accept_uni().await?;
    println!("   ... ok");
    println!("Opening configuration sender...");
    let mut configuration_up = connection.open_uni().await?;
    configuration_up.write(b"Hello there!").await?;
    println!("   ... ok");

    connection.close(VarInt::from_u32(0), &[]);

    connection.closed().await;

    Ok(())
}
