use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationUp, LocalStateSnapshot},
};
use quinn::{Connecting, Connection, ReadError, RecvStream, SendStream};
use tokio::{spawn, sync::broadcast};
use tokio_util::sync::CancellationToken;

use crate::{client::Client, dashboard::DashboardMessage, server::Server};

pub(crate) async fn accept_connection(
    connecting: Connecting,
    server: Server,
    ws: broadcast::Sender<DashboardMessage>,
) {
    let token = CancellationToken::new();
    match connecting.await {
        Ok(connection) => {
            let _ = ws.send(DashboardMessage::ConnectionEstablished {
                addr: connection.remote_address(),
                stable_id: connection.stable_id(),
            });
            println!("Connection established: {:?}", connection.remote_address());

            // Create client struct
            let client = Client::new(connection.clone(), ws.clone(), token.clone());
            server.add_client(client.clone()).await;
            match run_connection(client.clone(), connection, ws).await {
                Ok(()) => {
                    println!("Connection finished ok");
                }
                Err(e) => {
                    println!("Connection finished with error: {:?}", e);
                }
            }
            token.cancel();
            server.remove_client(client.id()).await;
        }
        Err(e) => {
            println!("Connection failed: {:?}", e);
        }
    }
}

async fn run_connection(
    client: Client,
    connection: Connection,
    ws: broadcast::Sender<DashboardMessage>,
) -> Result<()> {
    // Start sending heartbeat
    let heartbeat_client = client.clone();
    println!("Opening heartbeat channel");
    let heartbeat_channel = connection.open_uni().await?;
    let task_heartbeat = spawn(run_heartbeat(heartbeat_channel, heartbeat_client));

    // Start receiving configuration messages
    println!("Accepting configuration channel");
    let configuration_up_stream = connection.accept_uni().await?;
    let configuration_up_client = client.clone();
    let task_conf = spawn(run_configuration_up(
        configuration_up_stream,
        configuration_up_client,
    ));
    println!("Configuration channel opened");

    // Start receiving datagrams
    let task_datagram = spawn(run_datagram_up(connection.clone(), client.clone()));

    // TODO:
    // - rebroadcast configurations
    // - rebroadcast last datagrams

    let _ = ws.send(DashboardMessage::FullyConnected {
        stable_id: connection.stable_id(),
    });
    println!("Fully connected: {:?}", connection.remote_address());

    // Wait for some task to finish
    tokio::select! {
        _ = connection.closed() => {
            println!("Closed");
        },
        _ = task_conf => {
            println!("Configuration closed");
        },
        _ = task_datagram => {
            println!("Datagram closed");
        },
        _ = task_heartbeat => {
            println!("Heartbeat ended");
        },
    }

    // Report to dashboard and console
    ws.send(DashboardMessage::ConnectionClosed {
        stable_id: connection.stable_id(),
    })?;
    println!("Connection closed: {:?}", connection.remote_address());
    Ok(())
}

async fn run_heartbeat(mut heartbeat: SendStream, client: Client) {
    loop {
        match heartbeat.write_all(b"hello").await {
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
                Err(e) => match e {
                    ReadError::ConnectionLost(_) => {
                        break;
                    }
                    _ => {
                        if client.is_cancelled() { break; }
                        println!("Error reading configuration: {:?}", e);
                    }
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
                Ok(bytes) => match bincode::deserialize::<LocalStateSnapshot>(&bytes) {
                    Ok(message) => {
                        client.handle_datagram_up(message.clone()).await;
                    }
                    Err(e) => {
                        println!("Failed to decode configuration message: {:?}", e);
                    }
                },
                Err(e) => match e {
                    quinn::ConnectionError::ConnectionClosed(_)
                    | quinn::ConnectionError::ApplicationClosed(_)
                      => { break }
                    _ =>{
                        println!("Error reading datagram: {:?}", e);
                    }
                }
            }
        }
    }
}
