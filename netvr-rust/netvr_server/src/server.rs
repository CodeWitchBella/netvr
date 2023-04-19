use std::{collections::HashMap, sync::Arc};

use netvr_data::net::LocalStateSnapshot;
use tokio::{
    spawn,
    sync::{Mutex, RwLock},
};

use crate::client::Client;

pub(crate) type SnapshotChannel = tokio::sync::mpsc::Sender<(usize, LocalStateSnapshot)>;
type LatestSnaphots = Arc<RwLock<HashMap<usize, LocalStateSnapshot>>>;

#[derive(Clone)]
pub(crate) struct Server {
    clients: Arc<Mutex<HashMap<usize, Client>>>,
    latest_snapshots: LatestSnaphots,
    channel: SnapshotChannel,
}

impl Server {
    pub async fn start() -> Self {
        let latest_snapshots: LatestSnaphots = Arc::default();
        let channel = Self::receive_snapshots(latest_snapshots.clone()).await;
        Self {
            clients: Arc::default(),
            latest_snapshots,
            channel,
        }
    }

    async fn receive_snapshots(latest_snapshots: LatestSnaphots) -> SnapshotChannel {
        let (snapshot_channel, mut receiver) = tokio::sync::mpsc::channel(100);
        spawn(async move {
            loop {
                let Some((id, snapshot)) = receiver.recv().await else { break };
                let mut latest_snaphots = latest_snapshots.write().await;
                latest_snaphots.insert(id, snapshot);
            }
        });
        snapshot_channel
    }

    pub async fn send_snapshot(&self, id: usize, snapshot: LocalStateSnapshot) {
        if let Err(err) = self.channel.send((id, snapshot)).await {
            println!("Failed to send snapshot {:?}", err);
        }
    }

    pub async fn add_client(&self, client: Client) {
        let mut clients = self.clients.lock().await;
        clients.insert(client.id(), client);
    }

    pub async fn remove_client(&self, id: usize) {
        let mut clients = self.clients.lock().await;
        clients.remove(&id);
    }

    pub async fn read_latest_snapshots(&self) -> HashMap<usize, LocalStateSnapshot> {
        self.latest_snapshots.read().await.clone()
    }
}
