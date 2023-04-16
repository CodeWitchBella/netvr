use std::{collections::HashMap, net::SocketAddr};

use crate::{
    discovery_server::{init_discovery_server, run_discovery_server},
    server::Client,
};
use context::Context;
use netvr_data::{bincode, net};
use tokio::{
    net::{TcpListener, UdpSocket},
    spawn,
    time::sleep,
};

mod context;
mod discovery_server;
mod server;

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
    spawn(run_discovery_server(context.clone(), discovery_server));

    let mut clients: HashMap<SocketAddr, Client> = HashMap::new();
    let mut buf = [0u8; 65535];
    loop {
        let server_udp = context.server_udp().await;
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
    }
}
