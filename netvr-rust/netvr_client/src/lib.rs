use netvr_data::{bincode, net};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};
use tokio::{
    net::{TcpStream, UdpSocket},
    select,
};

/// Performs server discovery and returns a socket bound to correct address and port.
pub async fn connect() -> Result<(UdpSocket, TcpStream), Box<dyn std::error::Error>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    println!("Broadcasting as {:?}", socket.local_addr()?);

    loop {
        println!("Sending request");
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
                        let udp = UdpSocket::bind(SocketAddr::new(addr.ip(), 0)).await?;
                        udp.connect(addr).await?;
                        let tcp = TcpStream::connect(SocketAddr::new(addr.ip(), data.port)).await?;
                        return Ok((udp, tcp));
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        };
    }
}
