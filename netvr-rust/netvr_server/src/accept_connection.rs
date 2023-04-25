use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ClientId, ConfigurationDown, ConfigurationUp, Heartbeat, StateSnapshot},
    FramingError, RecvFrames, SendFrames,
};
use quinn::{Connecting, Connection};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{client::Client, dashboard::DashboardMessage, server::Server};

pub(crate) async fn accept_connection(
    connecting: Connecting,
    server: Server,
    ws: broadcast::Sender<DashboardMessage>,
    id: ClientId,
) {
    let token = CancellationToken::new();
    let mpsc = mpsc::unbounded_channel();

    // Create client struct
    let client = Client::new(
        ws.clone(),
        token.clone(),
        server.clone(),
        id,
        mpsc.0.clone(),
    );
    match run_connection(
        connecting,
        client.clone(),
        ws,
        server.clone(),
        mpsc.0,
        mpsc.1,
    )
    .await
    {
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
    configuration_channel_sender: mpsc::UnboundedSender<ConfigurationDown>,
    configuration_channel: mpsc::UnboundedReceiver<ConfigurationDown>,
) -> Result<()> {
    server.add_client(client.clone()).await?;

    // Accept connection and open channels
    let connection = connecting.await?;
    let heartbeat_channel = SendFrames::open(&connection, b"heartbee").await?;
    let configuration_up_stream = RecvFrames::open(&connection, b"configur").await?;
    let configuration_down_stream = SendFrames::open(&connection, b"confetti").await?;

    // Report to dashboard and console
    let _ = ws.send(DashboardMessage::ConnectionEstablished {
        id: client.id(),
        addr: connection.remote_address(),
    });
    println!("Connection established: {:?}", connection.remote_address());

    // Start sending heartbeat
    let task_heartbeat = run_heartbeat(heartbeat_channel);

    // Start receiving configuration messages
    let task_conf_up =
        run_configuration_up(configuration_up_stream, client.clone(), server.clone());

    // Start receiving datagrams
    let task_datagram = run_datagram_up(connection.clone(), client.clone(), server.clone());

    // Start sending configurations
    let task_conf_listen_change =
        run_configuration_listen_change(configuration_channel_sender, server.clone());
    let task_conf_down = run_configuration_down(configuration_down_stream, configuration_channel);

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
        _ = task_conf_up => {
            println!("Configuration closed");
        },
        _ = task_datagram => {
            println!("Datagram closed");
        },
        _ = task_heartbeat => {
            println!("Heartbeat ended");
        },
        res = task_conf_listen_change => {
            println!("Configuration listen change ended: {:?}", res);
        },
        res = task_conf_down => {
            println!("Configuration down ended: {:?}", res);
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
            Err(e) => match e {
                FramingError::ReadExactError(_)
                | FramingError::ConnectionError(quinn::ConnectionError::ConnectionClosed(_))
                | FramingError::ConnectionError(quinn::ConnectionError::ApplicationClosed(_))
                | FramingError::ConnectionError(quinn::ConnectionError::TimedOut) => break,
                _ => {
                    println!("Error reading configuration: {:?}", e);
                }
            },
        }
    }
}

async fn run_datagram_up(connection: Connection, client: Client, server: Server) {
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match bincode::deserialize::<StateSnapshot>(&bytes) {
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
                | quinn::ConnectionError::ApplicationClosed(_)
                | quinn::ConnectionError::TimedOut => break,
                _ => {
                    println!("Error reading datagram: {:?}", e);
                }
            },
        }
    }
}

async fn run_configuration_listen_change(
    channel: mpsc::UnboundedSender<ConfigurationDown>,
    server: Server,
) -> Result<()> {
    let mut conf = server.latest_configuration().await;
    loop {
        let val = conf.borrow().to_owned();
        println!("Sending configuration: {:?}", val);
        channel.send(ConfigurationDown::Snapshot(val))?;

        conf.changed().await?;
    }
}

async fn run_configuration_down(
    mut connection: SendFrames<ConfigurationDown>,
    mut channel: mpsc::UnboundedReceiver<ConfigurationDown>,
) -> Result<()> {
    loop {
        let Some(val) = channel.recv().await else { break; };
        println!("Sending configuration: {:?}", val);
        connection.write(&val).await?;
    }
    Ok(())
}
