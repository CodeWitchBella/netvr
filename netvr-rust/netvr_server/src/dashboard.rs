use std::{env, net::SocketAddr};

use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use netvr_data::{
    net::{ClientId, ConfigurationSnapshotSet, StateSnapshot},
    serde::{Deserialize, Serialize},
};
use tokio::sync::{broadcast, mpsc};
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

use crate::server::Server;

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
                println!("TODO: move some clients");
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
) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let split = ws.split();
    tokio::select! {
        _ = dashboard_send(split.0, broadcast_receiver, receiver) => {},
        _ = dashboard_send_configuration(server.clone(), sender.clone()) => {},
        _ = dashboard_receive(split.1, server, sender) => {},
    }
}

pub(crate) async fn serve_dashboard(tx: broadcast::Sender<DashboardMessage>, server: Server) {
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
            ws.on_upgrade(move |socket| dashboard_connected(socket, rx, server))
        });
    let files = warp::filters::fs::dir(dashboard.clone());
    let mut index = dashboard.clone();
    index.push("index.html");
    let routes = ws.or(files).or(warp::fs::file(index));
    warp::serve(routes).run(([0, 0, 0, 0], 13161)).await;
    println!("serving dashboard from {:?}", dashboard);
}
