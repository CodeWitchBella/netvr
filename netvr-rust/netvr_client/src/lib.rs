mod error;
mod quinn_connect;

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use error::Error;
use netvr_data::{
    bincode,
    net::{self, CalibrationSample, ConfigurationDown, ConfigurationUp, Heartbeat},
    RecvFrames, SendFrames,
};
use quinn::{Connection, Endpoint};
use tokio::{net::UdpSocket, select};

use crate::quinn_connect::quinn_connect;

pub struct NetVRConnection {
    pub endpoint: Endpoint,
    pub connection: Connection,
    pub heartbeat: RecvFrames<Heartbeat>,
    pub configuration_up: SendFrames<ConfigurationUp>,
    pub configuration_down: RecvFrames<ConfigurationDown>,
    pub calibration_up: SendFrames<CalibrationSample>,
}

/// Performs server discovery and returns a socket bound to correct address and
/// port.
pub async fn connect(log: fn(String) -> ()) -> Result<NetVRConnection, Error> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    log(format!("Broadcasting as {:?}", socket.local_addr()?));

    loop {
        log("Sending request".to_string());
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
                        return setup_connection(log, addr).await;
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        };
    }
}

async fn setup_connection(
    log: fn(String) -> (),
    addr: SocketAddr,
) -> Result<NetVRConnection, Error> {
    log(format!("Got valid response from {:?}", addr));
    let (endpoint, connection) = quinn_connect(addr).await?;
    log("Connection established.".to_string());
    log("Accepting heartbeat channel.".to_string());
    let heartbeat = RecvFrames::open(&connection, b"heartbee").await?;
    log("Heartbeat channel opened.".to_string());
    let configuration_up = SendFrames::open(&connection, b"configur").await?;
    log("Configuration up channel opened.".to_string());
    let configuration_down = RecvFrames::open(&connection, b"confetti").await?;
    log("Configuration down channel opened.".to_string());
    let calibration_up = SendFrames::open(&connection, b"calibrat").await?;
    log("Calibration up channel opened.".to_string());

    log("Channels opened.".to_string());

    Ok(NetVRConnection {
        endpoint,
        connection,
        heartbeat,
        configuration_up,
        configuration_down,
        calibration_up,
    })
}
