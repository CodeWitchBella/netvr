use anyhow::{anyhow, Result};
use netvr_data::{
    app::{AppDown, AppUp},
    bincode,
    net::{CalibrationSample, ClientId, ConfigurationDown, ConfigurationUp, DatagramUp, Heartbeat},
    FramingError, RecvFrames, SendFrames,
};
use quinn::{Connecting, Connection};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    app::{AppChannel, AppServerMessage},
    calibration_protocol::{CalibrationProtocolMessage, CalibrationSender},
    client::Client,
    dashboard::DashboardMessage,
    server::Server,
};

pub(crate) async fn accept_connection(
    connecting: Connecting,
    server: Server,
    ws: broadcast::Sender<DashboardMessage>,
    id: ClientId,
    calibration_sender: CalibrationSender,
    app_channel: AppChannel,
) {
    let token = CancellationToken::new();

    // Create client struct

    match run_connection(
        connecting,
        token.clone(),
        id,
        ws,
        server.clone(),
        calibration_sender,
        app_channel,
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
    token: CancellationToken,
    id: ClientId,
    ws: broadcast::Sender<DashboardMessage>,
    server: Server,
    calibration_sender: CalibrationSender,
    app_channel: AppChannel,
) -> Result<()> {
    // Accept connection and open channels
    let connection = connecting.await?;
    let heartbeat_channel = SendFrames::open(&connection, b"heartbee").await?;
    let configuration_up_stream = RecvFrames::open(&connection, b"configur").await?;
    let configuration_down_stream = SendFrames::open(&connection, b"confetti").await?;
    let calibration_up_stream: RecvFrames<CalibrationSample> =
        RecvFrames::open(&connection, b"calibrat").await?;
    let app_up_stream: RecvFrames<AppUp> = RecvFrames::open(&connection, b"app_up__").await?;
    let app_down_stream: SendFrames<AppDown> = SendFrames::open(&connection, b"app_down").await?;

    // Setup client
    let configuration_down_queue = mpsc::unbounded_channel();
    let app_down_queue = mpsc::unbounded_channel();
    let client = Client::new(
        ws.clone(),
        token.clone(),
        server.clone(),
        id,
        configuration_down_queue.0.clone(),
        app_down_queue.0.clone(),
        connection.clone(),
    );
    server.add_client(client.clone()).await?;
    let configuration_down_queue = configuration_down_queue.1;
    let app_down_queue = app_down_queue.1;

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
    let task_datagram = run_datagram_up(
        connection.clone(),
        client.clone(),
        server.clone(),
        app_channel.clone(),
    );

    // Start receiving and sending app messages
    let task_app_up = run_app_message_up(app_up_stream, app_channel, client.id());
    let task_app_down = run_app_message_down(app_down_stream, app_down_queue);

    // Start sending configurations
    let task_conf_listen_change = run_configuration_listen_change(client.clone(), server.clone());
    let task_conf_down =
        run_configuration_down(configuration_down_stream, configuration_down_queue);
    let task_calibration_up =
        run_calibration_up(client.id(), calibration_up_stream, calibration_sender);

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
        res = task_datagram => {
            println!("Datagram closed: {:?}", res);
        },
        _ = task_heartbeat => {
            println!("Heartbeat ended");
        },
        res = task_app_up => {
            println!("App up ended: {:?}", res);
        },
        res = task_app_down => {
            println!("App down ended: {:?}", res);
        },
        res = task_conf_listen_change => {
            println!("Configuration listen change ended: {:?}", res);
        },
        res = task_conf_down => {
            println!("Configuration down ended: {:?}", res);
        },
        res = task_calibration_up => {
            println!("Calibration up ended: {:?}", res);
        }
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

async fn run_app_message_up(
    mut connection: RecvFrames<AppUp>,
    app_channel: AppChannel,
    client_id: ClientId,
) -> Result<()> {
    loop {
        match connection.read().await {
            Ok(message) => {
                println!("Received app message: {:?}", message);
                app_channel.send(AppServerMessage::AppUp(client_id, message))?;
            }
            Err(e) => match e {
                FramingError::ReadExactError(_)
                | FramingError::ConnectionError(quinn::ConnectionError::ConnectionClosed(_))
                | FramingError::ConnectionError(quinn::ConnectionError::ApplicationClosed(_))
                | FramingError::ConnectionError(quinn::ConnectionError::TimedOut) => break,
                err => {
                    Err(err)?;
                }
            },
        }
    }
    Ok(())
}

async fn run_app_message_down(
    mut connection: SendFrames<AppDown>,
    mut channel: mpsc::UnboundedReceiver<AppDown>,
) -> Result<()> {
    loop {
        let Some(message) = channel.recv().await else { break; };
        connection.write(&message).await?;
    }
    Ok(())
}

async fn run_datagram_up(
    connection: Connection,
    client: Client,
    server: Server,
    app_channel: AppChannel,
) -> Result<()> {
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match bincode::deserialize::<DatagramUp>(&bytes) {
                Ok(message) => match message {
                    DatagramUp::State(message) => {
                        server.apply_snapshot(client.id(), message.clone()).await;
                        client.handle_recv_snapshot(message).await?;
                    }
                    DatagramUp::App(message) => {
                        app_channel.send(AppServerMessage::Datagram(client.id(), message))?;
                    }
                },
                Err(e) => {
                    println!("Failed to decode snapshot: {:?}", e);
                }
            },
            Err(e) => match e {
                quinn::ConnectionError::ConnectionClosed(_)
                | quinn::ConnectionError::ApplicationClosed(_)
                | quinn::ConnectionError::TimedOut => break,
                _ => {
                    Err(anyhow!("Error reading datagram: {:?}", e))?;
                }
            },
        }
    }
    Ok(())
}

async fn run_configuration_listen_change(client: Client, server: Server) -> Result<()> {
    let mut conf = server.latest_configuration().await;
    loop {
        let val = conf.borrow().to_owned();
        client.send_configuration_down(ConfigurationDown::Snapshot(val))?;

        conf.changed().await?;
    }
}

async fn run_configuration_down(
    mut connection: SendFrames<ConfigurationDown>,
    mut channel: mpsc::UnboundedReceiver<ConfigurationDown>,
) -> Result<()> {
    loop {
        let Some(val) = channel.recv().await else { break; };
        println!("Sending configuration: <snip>");
        connection.write(&val).await?;
    }
    Ok(())
}

async fn run_calibration_up(
    client_id: ClientId,
    mut connection: RecvFrames<CalibrationSample>,
    channel: CalibrationSender,
) -> Result<()> {
    loop {
        let val = connection.read().await?;
        println!("Forwarding calibration: {:?}", val);
        channel
            .send(CalibrationProtocolMessage::Sample {
                client: client_id,
                sample: val,
            })
            .map_err(|err| anyhow!("Failed to forward calibration message: {:?}", err))?;
    }
}
