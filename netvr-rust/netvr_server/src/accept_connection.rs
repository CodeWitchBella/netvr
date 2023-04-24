use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ClientId, ConfigurationUp, Heartbeat, LocalStateSnapshot},
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
    id: ClientId,
) {
    let token = CancellationToken::new();

    // Create client struct
    let client = Client::new(ws.clone(), token.clone(), server.clone(), id);
    match run_connection(connecting, client.clone(), ws, server.clone()).await {
        Ok(()) => {
            println!("Connection finished ok");
        }
        Err(e) => {
            println!("Connection finished with error: {:?}", e);
        }
    }
    token.cancel();
    server.remove_client(id).await;
}

async fn run_connection(
    connecting: Connecting,
    client: Client,
    ws: broadcast::Sender<DashboardMessage>,
    server: Server,
) -> Result<()> {
    server.add_client(client.clone()).await?;

    // Accept connection and open channels
    let connection = connecting.await?;
    let heartbeat_channel = SendFrames::open(&connection, b"heartbee").await?;
    let configuration_up_stream = RecvFrames::open(&connection, b"configur").await?;

    // Report to dashboard and console
    let _ = ws.send(DashboardMessage::ConnectionEstablished {
        id: client.id(),
        addr: connection.remote_address(),
    });
    println!("Connection established: {:?}", connection.remote_address());

    // Start sending heartbeat
    let task_heartbeat = run_heartbeat(heartbeat_channel);

    // Start receiving configuration messages
    let task_conf = run_configuration_up(configuration_up_stream, client.clone(), server.clone());

    // Start receiving datagrams
    let task_datagram = run_datagram_up(connection.clone(), client.clone(), server.clone());

    // TODO:
    // - rebroadcast configurations
    // - rebroadcast last datagrams

    let _ = ws.send(DashboardMessage::FullyConnected { id: client.id() });
    println!("Fully connected: {:?}", connection.remote_address());

    // Wait for some task to finish
    tokio::select! {
        _ = client.cancelled() => {
            println!("Cancelled");
        },
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
    ws.send(DashboardMessage::ConnectionClosed { id: client.id() })?;
    println!("Connection closed: {:?}", connection.remote_address());
    Ok(())
}

async fn run_heartbeat(mut heartbeat: SendFrames<Heartbeat>) {
    loop {
        match heartbeat.write(&Heartbeat::default()).await {
            Ok(v) => {
                println!("Sent heartbeat {:?}", v);
            }
            Err(e) => {
                println!("Error sending heartbeat: {:?}", e);
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await
    }
}

async fn run_configuration_up(
    mut configuration_up: RecvFrames<ConfigurationUp>,
    client: Client,
    server: Server,
) {
    loop {
        match configuration_up.read().await {
            Ok(message) => {
                if let ConfigurationUp::ConfigurationSnapshot(snapshot) = &message {
                    server
                        .apply_configuration(client.id(), snapshot.clone())
                        .await;
                }
                client.handle_configuration_up(message).await;
            }
            Err(e) => {
                if client.is_cancelled() {
                    break;
                }
                println!("Error reading configuration: {:?}", e);
            }
        }
    }
}

async fn run_datagram_up(connection: Connection, client: Client, server: Server) {
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match bincode::deserialize::<LocalStateSnapshot>(&bytes) {
                Ok(message) => {
                    if let Err(err) = client
                        .handle_recv_snapshot(message.clone(), &connection)
                        .await
                    {
                        println!("Failed to handle snapshot: {:?}", err);
                        return;
                    }
                    server.apply_snapshot(client.id(), message).await;
                }
                Err(e) => {
                    println!("Failed to decode snapshot: {:?}", e);
                }
            },
            Err(e) => match e {
                quinn::ConnectionError::ConnectionClosed(_)
                | quinn::ConnectionError::ApplicationClosed(_) => break,
                _ => {
                    println!("Error reading datagram: {:?}", e);
                }
            },
        }
    }
}
