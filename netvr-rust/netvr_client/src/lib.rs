mod quinn_connect;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use netvr_data::{bincode, net};
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use tokio::{net::UdpSocket, select};

use crate::quinn_connect::quinn_connect;

pub struct NetVRConnection {
    pub endpoint: Endpoint,
    pub connection: Connection,
    pub heartbeat: RecvStream,
    pub configuration_up: SendStream,
}

/// Performs server discovery and returns a socket bound to correct address and
/// port.
pub async fn connect() -> Result<NetVRConnection, Box<dyn std::error::Error>> {
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
                        println!("[discovery] Got valid response from {:?}", addr);
                        let (endpoint, connection) = quinn_connect(addr).await?;
                        println!("[discovery] Connection established.");
                        let heartbeat = connection.accept_uni().await?;
                        let mut configuration_up = connection.open_uni().await?;
                        configuration_up.write(&bincode::serialize(&net::ConfigurationUp::Hello)?).await ?;
                        println!("[discovery] Channels opened.");

                        return Ok(NetVRConnection {
                            endpoint,
                            connection,
                            heartbeat,
                            configuration_up,
                        });
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        };
    }
}
