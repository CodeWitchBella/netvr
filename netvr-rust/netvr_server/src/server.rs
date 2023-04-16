use crate::{client::Client, dashboard::DashboardMessage};
use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationUp, DatagramUp},
};
use quinn::{Connecting, Connection, RecvStream};
use std::sync::Arc;
use tokio::{
    spawn,
    sync::{broadcast, Mutex},
};

pub struct Server {
    clients: Vec<Client>,
}

impl Server {
    pub fn new() -> Self {
        Self { clients: vec![] }
    }
}

impl Client {
    pub(crate) async fn accept_connection(
        connecting: Connecting,
        server: Arc<Mutex<Server>>,
        ws: broadcast::Sender<DashboardMessage>,
    ) -> Result<()> {
        match connecting.await {
            Ok(connection) => {
                let _ = ws.send(DashboardMessage::ConnectionEstablished(
                    connection.remote_address(),
                    connection.stable_id(),
                ));
                println!("Connection established: {:?}", connection);
                let client = Client::new(connection.clone()).await?;
                server.lock().await.clients.push(client.clone());

                let configuration_up_stream = connection.accept_uni().await?;
                let configuration_up_client = client.clone();
                spawn(run_configuration_up(
                    configuration_up_stream,
                    configuration_up_client,
                ));

                run_datagram_up(connection, client).await;
            }
            Err(e) => {
                println!("Connection failed: {:?}", e);
            }
        }
        Ok(())
    }
}

async fn run_configuration_up(mut configuration_up: RecvStream, client: Client) {
    let mut buf = [0u8; 65535];
    loop {
        match configuration_up.read(&mut buf).await {
            Ok(amt) => {
                if let Some(amt) = amt {
                    match bincode::deserialize::<ConfigurationUp>(&buf[..amt]) {
                        Ok(message) => {
                            client.handle_configuration_up(message).await;
                        }
                        Err(e) => {
                            println!("Failed to decode configuration message: {:?}", e);
                        }
                    }
                } else {
                    println!("Connection closed");
                    return;
                }
            }
            Err(e) => {
                println!("Error reading configuration: {:?}", e);
            }
        }
    }
}

async fn run_datagram_up(connection: Connection, client: Client) {
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match bincode::deserialize::<DatagramUp>(&bytes) {
                Ok(message) => {
                    client.handle_datagram_up(message).await;
                }
                Err(e) => {
                    println!("Failed to decode configuration message: {:?}", e);
                }
            },
            Err(e) => {
                println!("Error reading datagram: {:?}", e);
            }
        }
    }
}
