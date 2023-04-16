use crate::{client::Client, dashboard::DashboardMessage, server::Server};
use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationUp, DatagramUp},
};
use quinn::{Connecting, Connection, RecvStream, SendStream};
use std::sync::Arc;
use tokio::{
    spawn,
    sync::{broadcast, Mutex},
    time::timeout,
};
use tokio_util::sync::CancellationToken;

pub(crate) async fn accept_connection(
    connecting: Connecting,
    server: Arc<Mutex<Server>>,
    ws: broadcast::Sender<DashboardMessage>,
) -> Result<()> {
    let token = CancellationToken::new();
    let _ = token.clone().drop_guard();
    match connecting.await {
        Ok(connection) => {
            let _ = ws.send(DashboardMessage::ConnectionEstablished(
                connection.remote_address(),
                connection.stable_id(),
            ));
            println!("Connection established: {:?}", connection.remote_address());

            // Create client struct
            let client = Client::new(connection.clone(), ws.clone(), token.clone()).await?;
            server.lock().await.clients.push(client.clone());

            // Start sending heartbeat
            let heartbeat_client = client.clone();
            let task_heartbeat = spawn(run_heartbeat(
                connection.open_uni().await?,
                heartbeat_client,
            ));

            // Start receiving configuration messages
            let configuration_up_stream =
                timeout(tokio::time::Duration::from_secs(5), connection.accept_uni()).await??;
            let configuration_up_client = client.clone();
            let task_conf = spawn(run_configuration_up(
                configuration_up_stream,
                configuration_up_client,
            ));

            // Start receiving datagrams
            let task_datagram = spawn(run_datagram_up(connection.clone(), client));

            println!("Fully connected: {:?}", connection.remote_address());

            // Wait for some task to finish
            tokio::select! {
                _ = connection.closed() => {},
                _ = task_conf => {},
                _ = task_datagram => {},
                _ = task_heartbeat => {},
            }
            // and cancel all other tasks
            token.cancel();

            // Report to dashboard and console
            ws.send(DashboardMessage::ConnectionClosed)?;
            println!("Connection closed: {:?}", connection.remote_address());
        }
        Err(e) => {
            println!("Connection failed: {:?}", e);
        }
    }
    Ok(())
}

async fn run_heartbeat(mut heartbeat: SendStream, client: Client) {
    loop {
        match heartbeat.write_all(b"").await {
            Ok(v) => {
                println!("Sent heartbeat {:?}", v);
            }
            Err(e) => {
                println!("Error sending heartbeat: {:?}", e);
            }
        };
        tokio::select! {
            _ = client.cancelled() => {
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
        };
    }
}

async fn run_configuration_up(mut configuration_up: RecvStream, client: Client) {
    let mut buf = [0u8; 65535];
    loop {
        tokio::select! {
            _ = client.cancelled() => {
                break;
            }
            message = configuration_up.read(&mut buf) => match message {
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
                        break;
                    }
                }
                Err(e) => {
                    println!("Error reading configuration: {:?}", e);
                }
            }
        }
    }
}

async fn run_datagram_up(connection: Connection, client: Client) {
    loop {
        tokio::select! {
            _ = client.cancelled() => {
                break;
            }
            datagram = connection.read_datagram() => match datagram {
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
}
