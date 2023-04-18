use std::{env, net::SocketAddr};

use anyhow::{anyhow, Result};
use futures_util::SinkExt;
use netvr_data::{
    net::LocalStateSnapshot,
    serde::{Deserialize, Serialize},
};
use tokio::sync::broadcast;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum DashboardMessage {
    Binary(Vec<u8>),
    #[serde(rename_all = "camelCase")]
    ConnectionEstablished {
        addr: SocketAddr,
        stable_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    FullyConnected {
        stable_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    ConnectionClosed {
        stable_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    DatagramUp {
        stable_id: usize,
        message: LocalStateSnapshot,
    },
}

async fn dashboard_connected(
    mut ws: WebSocket,
    mut receiver: broadcast::Receiver<DashboardMessage>,
) {
    loop {
        match receiver.recv().await {
            Ok(msg) => match ws
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
            },
            Err(err) => match err {
                broadcast::error::RecvError::Closed => {
                    println!("dashboard connection closed");
                    break;
                }
                broadcast::error::RecvError::Lagged(n) => {
                    let _ = ws
                        .send(Message::text(format!("failed to forward {} messages", n)))
                        .await;
                }
            },
        }
        // ```
        // let Some(msg) = ws.next().await else { break };
        // let Ok(msg) = msg else { println!("Failed to receive message: {:?}", msg.err()); continue };
        // if let Err(err) = ws.send(msg).await {
        //     println!("failed to send message: {}", err);
        // }
        // ```
    }
}

pub(crate) async fn serve_dashboard(tx: broadcast::Sender<DashboardMessage>) {
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
            ws.on_upgrade(move |socket| dashboard_connected(socket, rx))
        });
    let files = warp::filters::fs::dir(dashboard);
    let routes = ws.or(files);
    warp::serve(routes).run(([0, 0, 0, 0], 13161)).await;
}
