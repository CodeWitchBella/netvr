use crate::{
    client::Client,
    discovery_server::{init_discovery_server, run_discovery_server},
    my_socket::MySocket,
    quinn_server::make_server_endpoint,
    server::Server,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::{net::UdpSocket, spawn, sync::Mutex};

mod client;
mod discovery_server;
mod my_socket;
mod quinn_server;
mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let server_udp = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let endpoint = make_server_endpoint(MySocket::new(server_udp.clone()))?;
    let server_port = endpoint.local_addr()?.port();

    println!("Server port: {:?}", server_port);

    let discovery_server = init_discovery_server().await?;
    spawn(run_discovery_server(server_udp.clone(), discovery_server));

    let server = Arc::new(Mutex::new(Server::new()));
    loop {
        if let Some(connecting) = endpoint.accept().await {
            spawn(Client::accept_connection(connecting, server.clone()));
        }
    }
}
