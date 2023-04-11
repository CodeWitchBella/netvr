use crate::context::Context;
use netvr_data::{bincode, net};
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

/// Initializes the discovery server and returns the data needed to run it.
pub(crate) async fn init_discovery_server(
    context: &Context,
) -> Result<(UdpSocket, Vec<u8>), Box<dyn std::error::Error>> {
    let discovery_socket = UdpSocket::bind("0.0.0.0:13161").await?;
    println!(
        "[discovery] Discovery address: {:?}",
        discovery_socket.local_addr()?
    );
    let multi_addr = Ipv4Addr::new(234, 2, 2, 2);
    let inter = Ipv4Addr::new(0, 0, 0, 0);
    discovery_socket.join_multicast_v4(multi_addr, inter)?;
    let discovery_response = bincode::serialize(&net::DiscoveryResponse::new({
        let server_tcp = context.server_tcp().await;
        server_tcp.local_addr()?.port()
    }))?;
    Ok((discovery_socket, discovery_response))
}

/// Starts the discovery server and runs it forever.
pub(crate) async fn run_discovery_server(context: &Context, data: (UdpSocket, Vec<u8>)) {
    let (discovery_socket, discovery_response) = data;
    let mut buf = [0u8; 65535];
    loop {
        match discovery_socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                println!("[discovery] received {} bytes from {:?}", amt, src);
                let eq = buf[0..amt].eq("netvr".as_bytes());
                println!("[discovery] eq: {}", eq);
                if eq {
                    println!("[discovery] Sending response");

                    let server_udp = context.server_udp().await;

                    if let Err(e) = server_udp.send_to(&discovery_response, src).await {
                        println!("[discovery] send_to err = {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("[discovery] recv_from err = {:?}", e);
            }
        };
    }
}
