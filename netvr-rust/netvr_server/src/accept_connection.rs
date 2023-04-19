use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationUp, Heartbeat, LocalStateSnapshot},
    RecvFrames, SendFrames,
};
use quinn::{Connecting, Connection};
use tokio::{spawn, sync::broadcast};
use tokio_util::sync::CancellationToken;

use crate::{client::Client, dashboard::DashboardMessage, server::Server};

pub(crate) async fn accept_connection(
    connecting: Connecting,
    server: Server,
    ws: broadcast::Sender<DashboardMessage>,
) {
    let token = CancellationToken::new();

    // Create client struct
    let client = Client::new(ws.clone(), token.clone(), server.clone());
    server.add_client(client.clone()).await;
    match run_connection(connecting, client.clone(), ws, server.clone()).await {
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

async fn run_connection(
    connecting: Connecting,
    mut client: Client,
    ws: broadcast::Sender<DashboardMessage>,
    server: Server,
) -> Result<()> {
    // Accept connection and open channels
    let connection = connecting.await?;
    client.set_id(connection.stable_id());
    let heartbeat_channel = SendFrames::open(&connection, b"heartbee").await?;
    let configuration_up_stream = RecvFrames::open(&connection, b"configur").await?;

    // Report to dashboard and console
    let _ = ws.send(DashboardMessage::ConnectionEstablished {
        addr: connection.remote_address(),
        stable_id: connection.stable_id(),
    });
    println!("Connection established: {:?}", connection.remote_address());

    // Start sending heartbeat
    let task_heartbeat = spawn(run_heartbeat(heartbeat_channel, client.clone()));

    // Start receiving configuration messages
    let task_conf = spawn(run_configuration_up(
        configuration_up_stream,
        client.clone(),
    ));

    // Start receiving datagrams
    let task_datagram = spawn(run_datagram_up(
        connection.clone(),
        client.clone(),
        server.clone(),
    ));

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

async fn run_heartbeat(mut heartbeat: SendFrames<Heartbeat>, client: Client) {
    loop {
        match heartbeat.write(&Heartbeat::default()).await {
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

async fn run_configuration_up(mut configuration_up: RecvFrames<ConfigurationUp>, client: Client) {
    loop {
        tokio::select! {
            _ = client.cancelled() => {
                break;
            }
            message = configuration_up.read() => match message {
                Ok(message) => {
                    client.handle_configuration_up(message).await;
                }
                Err(e) => {
                    if client.is_cancelled() { break; }
                    println!("Error reading configuration: {:?}", e);
                }
            }
        }
    }
}

async fn run_datagram_up(connection: Connection, client: Client, server: Server) {
    loop {
        tokio::select! {
            _ = client.cancelled() => {
                break;
            }
            datagram = connection.read_datagram() => match datagram {
                Ok(bytes) => match bincode::deserialize::<LocalStateSnapshot>(&bytes) {
                    Ok(message) => {
                        client.handle_datagram_up(message.clone()).await;
                        server.send_snapshot(client.id(), message).await;
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
