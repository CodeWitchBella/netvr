use crate::discovery_server::{init_discovery_server, run_discovery_server};
use context::Context;
use tokio::{
    net::{TcpListener, UdpSocket},
    spawn,
    time::sleep,
};

mod context;
mod discovery_server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_udp = UdpSocket::bind("0.0.0.0:0").await?;
    let server_tcp = TcpListener::bind("0.0.0.0:0").await?;

    println!(
        "Server address: {:?} (TCP) {:?} (UDP)",
        server_tcp.local_addr()?,
        server_udp.local_addr()?
    );

    let context = Context::new(server_udp, server_tcp);
    let discovery_server = init_discovery_server(&context).await?;
    spawn(async move {
        run_discovery_server(&context, discovery_server).await;
    });

    loop {
        sleep(std::time::Duration::from_secs(1)).await;
    }
}
