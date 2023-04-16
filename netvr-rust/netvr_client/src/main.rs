use netvr_client::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello there! I'm looking for NetVR devices...");
    let (_endpoint, connection) = connect().await?;
    println!("  remote_address: {:?}", connection.remote_address());
    println!("  local: {:?}", connection.local_ip());

    Ok(())
}
