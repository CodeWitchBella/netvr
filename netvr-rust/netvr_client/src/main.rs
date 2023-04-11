use netvr_client::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello there! I'm looking for NetVR devices...");
    let (udp, tcp) = connect().await?;
    println!("UDP:");
    println!("  server: {:?}", udp.peer_addr()?);
    println!("  local: {:?}", udp.local_addr()?);
    println!("TCP:");
    println!("  server: {:?}", tcp.peer_addr()?);
    println!("  local: {:?}", tcp.local_addr()?);

    Ok(())
}
