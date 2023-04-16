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

    /*
    let mut clients: HashMap<SocketAddr, Client> = HashMap::new();
    let mut buf = [0u8; 65535];
    loop {
        match server_udp.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                let client = clients.get(&src);
                let datagram = bincode::deserialize::<net::UdpDatagramUp>(&buf[..amt]);
                match datagram {
                    Ok(datagram) => {
                        if let Some(client) = client {
                            spawn(client.clone().handle_udp(datagram));
                        } else {
                            println!("[udp] New client: {:?}", src);
                            let client = Client::new(src);
                            clients.insert(src, client.clone());
                            spawn(client.clone().handle_udp(datagram));
                        }
                    }
                    Err(e) => {
                        println!("[udp] failed to decode UDP message: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("[udp] recv_from err = {:?}", e);
            }
        };
    } */
    let server = Arc::new(Mutex::new(Server::new()));
    loop {
        if let Some(connecting) = endpoint.accept().await {
            let server = server.clone();
            spawn(async move {
                let client = Client::accept_connection(connecting, server).await;
                println!("New connection: {:?}", client);

                // TODO: store the client
            });
        }
    }
}
