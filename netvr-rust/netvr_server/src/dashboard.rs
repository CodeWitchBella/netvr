use std::{env, net::SocketAddr};

use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use netvr_data::{
    net::{ClientId, ConfigurationDown, ConfigurationSnapshotSet, StateSnapshot},
    serde::{Deserialize, Serialize},
};
use tokio::sync::{broadcast, mpsc};
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::{
    calibration_protocol::{CalibrationProtocolMessage::Begin, CalibrationSender},
    server::Server,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum DashboardMessage {
    Binary(Vec<u8>),
    #[serde(rename_all = "camelCase")]
    ConnectionEstablished {
        id: ClientId,
        addr: SocketAddr,
    },
    #[serde(rename_all = "camelCase")]
    FullyConnected {
        id: ClientId,
    },
    #[serde(rename_all = "camelCase")]
    ConnectionClosed {
        id: ClientId,
    },
    #[serde(rename_all = "camelCase")]
    DatagramUp {
        id: ClientId,
        message: StateSnapshot,
    },
    #[serde(rename_all = "camelCase")]
    ConfigurationSnapshotChanged {
        value: ConfigurationSnapshotSet,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum DashboardMessageRecv {
    MoveSomeClients,
    KeepAlive,
    Init,
    CalibrateByHeadsetPosition,
    #[serde(rename_all = "camelCase")]
    TriggerHapticImpulse {
        client_id: ClientId,
        subaction_path: String,
    },
    #[serde(rename_all = "camelCase")]
    ResetCalibration {
        client_id: ClientId,
    },
    #[serde(rename_all = "camelCase")]
    StartCalibration {
        leader_id: ClientId,
        leader_subaction_path: String,
        follower_id: ClientId,
        follower_subaction_path: String,
        sample_count: usize,
    },
}

async fn dashboard_send(
    mut ws: SplitSink<WebSocket, warp::ws::Message>,
    mut receiver: broadcast::Receiver<DashboardMessage>,
    mut reply: mpsc::UnboundedReceiver<DashboardMessage>,
) {
    loop {
        let msg = tokio::select! {
            res = receiver.recv() => match res {
                Ok(msg) => msg,
                Err(err) => match err {
                    broadcast::error::RecvError::Closed => {
                        println!("dashboard connection closed");
                        break;
                    }
                    broadcast::error::RecvError::Lagged(n) => {
                        let _ = ws
                            .send(Message::text(format!("failed to forward {} messages", n)))
                            .await;
                        continue
                    }
                }
            },
            msg = reply.recv() => match msg {
                Some(msg) => msg,
                None => { continue }
            },
        };

        match ws
            .send(if let DashboardMessage::Binary(b) = msg {
                Message::binary(b)
            } else {
                Message::text(serde_json::to_string(&msg).unwrap()) // TODO<- remove unwrap
            })
            .await
        {
            Ok(_) => {}
            Err(err) => {
                println!("failed to send message: {}", err);
                break;
            }
        }
    }
}

async fn dashboard_receive(
    mut ws: SplitStream<WebSocket>,
    server: Server,
    reply: mpsc::UnboundedSender<DashboardMessage>,
    calibration_sender: CalibrationSender,
) {
    loop {
        let Some(val) = ws.next().await else { break; };
        let val = match val {
            Ok(val) => val,
            Err(error) => {
                println!("Error receiving message from websocket: {}", error);
                break;
            }
        };
        let Ok(val_str) = val.to_str() else {
            println!("TODO: binary messages");
            continue;
        };

        let val = serde_json::from_str::<DashboardMessageRecv>(val_str);
        let val = match val {
            Ok(val) => val,
            Err(error) => {
                println!("Failed to parse message \"{}\": {:?}", val_str, error);
                continue;
            }
        };
        match val {
            DashboardMessageRecv::MoveSomeClients => {
                if let Some(client) = server.get_first_client().await {
                    if let Err(err) = client.send_configuration_down(ConfigurationDown::StagePose(
                        netvr_data::Pose {
                            position: netvr_data::Vec3 {
                                x: 0.,
                                y: 0.,
                                z: 1.,
                            },
                            orientation: netvr_data::Quaternion {
                                x: 0.,
                                y: 0.,
                                z: 0.,
                                w: 1.,
                            },
                        },
                    )) {
                        println!("Failed to send configuration down: {}", err);
                    }
                } else {
                    println!("... no clients");
                }
            }
            DashboardMessageRecv::KeepAlive => {}
            DashboardMessageRecv::Init => {
                let Ok(_) = reply.send(DashboardMessage::ConfigurationSnapshotChanged {
                    value: {
                        let watch = server.latest_configuration().await;
                        let value = watch.borrow().to_owned();
                        value
                    },
                }) else { return; };
            }
            DashboardMessageRecv::StartCalibration {
                leader_id: a_id,
                leader_subaction_path: a_path,
                follower_id: b_id,
                follower_subaction_path: b_path,
                sample_count,
            } => {
                if let Err(err) = calibration_sender.send(Begin {
                    clients: ((a_id, a_path), (b_id, b_path)),
                    sample_count,
                }) {
                    println!("Failed to send calibration request: {}", err);
                }
            }
            DashboardMessageRecv::CalibrateByHeadsetPosition => {
                println!("TODO: sync by head position")
            }
            DashboardMessageRecv::ResetCalibration { client_id } => {
                println!("TODO: reset calibration for {}", client_id)
            }
            DashboardMessageRecv::TriggerHapticImpulse {
                client_id,
                subaction_path,
            } => {
                println!(
                    "TODO: trigger haptic impulse for {} {}",
                    client_id, subaction_path
                )
            }
        }
    }
}

async fn dashboard_send_configuration(
    server: Server,
    reply: mpsc::UnboundedSender<DashboardMessage>,
) {
    let mut conf = server.latest_configuration().await;
    loop {
        if let Err(error) = conf.changed().await {
            println!("Failed to wait for configuration change: {}", error);
            return;
        }

        if let Err(error) = reply.send(DashboardMessage::ConfigurationSnapshotChanged {
            value: conf.borrow().clone(),
        }) {
            println!("Failed to send configuration snapshot: {}", error);
            return;
        }
    }
}

async fn dashboard_connected(
    ws: WebSocket,
    broadcast_receiver: broadcast::Receiver<DashboardMessage>,
    server: Server,
    calibration_sender: CalibrationSender,
) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let split = ws.split();
    tokio::select! {
        _ = dashboard_send(split.0, broadcast_receiver, receiver) => {},
        _ = dashboard_send_configuration(server.clone(), sender.clone()) => {},
        _ = dashboard_receive(split.1, server, sender, calibration_sender) => {},
    }
}

pub(crate) async fn serve_dashboard(
    tx: broadcast::Sender<DashboardMessage>,
    server: Server,
    calibration_sender: CalibrationSender,
) {
    let exe_parent: Result<_> = match std::env::current_exe() {
        Ok(p) => match p.parent() {
            Some(p) => anyhow::Result::Ok(p.to_owned()),
            None => anyhow::Result::Err(anyhow!("failed to get parent of current exe path")),
        },
        Err(e) => anyhow::Result::Err(e.into()),
    };
    let path = match exe_parent {
        Ok(p) => p,
        Err(e) => {
            println!("failed to get current exe path, falling back to cwd: {}", e);
            match env::current_dir() {
                Ok(p) => p.as_path().to_owned(),
                Err(e) => {
                    panic!("failed to get current dir: {}", e);
                }
            }
        }
    };
    let mut dashboard = path.to_path_buf();
    dashboard.push("dashboard");
    if !dashboard.exists() {
        dashboard.pop();
        dashboard.pop();
        dashboard.pop();
        dashboard.pop();
        dashboard.push("netvr-dashboard");
        dashboard.push("dist");
        if !dashboard.exists() {
            panic!("failed to find dashboard");
        }
    }
    let ws = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let rx = tx.subscribe();
            let server = server.clone();
            let calibration_sender = calibration_sender.clone();
            ws.on_upgrade(move |socket| dashboard_connected(socket, rx, server, calibration_sender))
        });
    let files = warp::filters::fs::dir(dashboard.clone());
    let mut index = dashboard.clone();
    index.push("index.html");
    let routes = ws.or(files).or(warp::fs::file(index));
    warp::serve(routes).run(([0, 0, 0, 0], 13161)).await;
    println!("serving dashboard from {:?}", dashboard);
}
