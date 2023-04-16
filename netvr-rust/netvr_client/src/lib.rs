mod quinn_connect;

use netvr_data::{bincode, net};
use quinn::{Connection, Endpoint};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};
use tokio::{net::UdpSocket, select};

use crate::quinn_connect::quinn_connect;

/// Performs server discovery and returns a socket bound to correct address and port.
pub async fn connect() -> Result<(Endpoint, Connection), Box<dyn std::error::Error>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    println!("[discovery] Broadcasting as {:?}", socket.local_addr()?);

    loop {
        println!("[discovery] Sending request");
        socket
            .send_to(
                "netvr".as_bytes(),
                SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 13161),
            )
            .await?;

        let mut recv_buff: [u8; 10] = [0; 10];

        select! {
            () = tokio::time::sleep(Duration::from_millis(500)) => {},
            rec = socket.recv_from(recv_buff.as_mut()) => {
                let (n, addr) = rec?;
                if let Ok(data) = bincode::deserialize::<net::DiscoveryResponse>(&recv_buff[..n]) {
                    if !data.validate_header() {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    } else {
                        println!("[discovery] Got response from {:?}", addr);
                        let (endpoint, connection) = quinn_connect(addr).await?;
                        println!("[discovery] Connection established {:?}", addr);
                        return Ok((endpoint, connection));
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        };
    }
}
