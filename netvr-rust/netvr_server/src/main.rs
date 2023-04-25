use std::sync::Arc;

use anyhow::Result;
use tokio::{net::UdpSocket, spawn, sync::broadcast};

use crate::{
    accept_connection::accept_connection,
    dashboard::{serve_dashboard, DashboardMessage},
    discovery_server::{init_discovery_server, run_discovery_server},
    my_socket::MySocket,
    quinn_server::make_server_endpoint,
    server::Server,
};

mod accept_connection;
mod client;
mod dashboard;
mod discovery_server;
mod my_socket;
mod quinn_server;
mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let server_udp = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let endpoint = make_server_endpoint(MySocket::new(server_udp.clone()))?;
    let server_port = endpoint.local_addr()?.port();
    let (tx, mut rx) = broadcast::channel::<DashboardMessage>(16);

    println!("Server port: {:?}", server_port);

    let discovery_server = init_discovery_server().await?;
    let discovery = spawn(run_discovery_server(server_udp.clone(), discovery_server));
    let server = Server::start().await;
    let dashboard = spawn(serve_dashboard(tx.clone(), server.clone()));

    let connections = spawn(async move {
        let mut id_generator: u32 = 0;
        loop {
            if let Some(connecting) = endpoint.accept().await {
                let tx = tx.clone();
                let server = server.clone();
                id_generator += 1;
                spawn(accept_connection(connecting, server, tx, id_generator));
            }
        }
    });

    // black-hole all the messages so that channel does not get closed
    spawn(async move {
        loop {
            let _ = rx.recv().await;
        }
    });

    dashboard.await?;
    discovery.await?;
    connections.await?;
    Ok(())
}
