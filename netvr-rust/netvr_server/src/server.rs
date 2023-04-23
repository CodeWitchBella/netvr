use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use netvr_data::net::{
    ClientId, LocalConfigurationSnapshot, LocalStateSnapshot, RemoteConfigurationSnapshot,
    RemoteStateSnapshot,
};
use tokio::{
    spawn,
    sync::{Mutex, RwLock},
};

use crate::client::Client;

#[derive(Debug)]
enum ServerChange {
    AddClient(ClientId),
    SetSnapshot(ClientId, LocalStateSnapshot),
    SetConfiguration(ClientId, LocalConfigurationSnapshot<()>),
    RemoveClient(ClientId),
}

type ServerChannel = tokio::sync::mpsc::Sender<ServerChange>;
type LatestSnaphots = Arc<RwLock<RemoteStateSnapshot>>;
type LatestConfigurations = Arc<RwLock<RemoteConfigurationSnapshot>>;

#[derive(Clone)]
pub(crate) struct Server {
    clients: Arc<Mutex<HashMap<ClientId, Client>>>,
    latest_snapshots: LatestSnaphots,
    latest_configurations: LatestConfigurations,
    channel: ServerChannel,
}

impl Server {
    pub async fn start() -> Self {
        let latest_snapshots: LatestSnaphots = Arc::default();
        let latest_configurations: LatestConfigurations = Arc::default();
        let channel = Self::receive(latest_snapshots.clone(), latest_configurations.clone()).await;
        Self {
            clients: Arc::default(),
            latest_snapshots,
            latest_configurations,
            channel,
        }
    }

    async fn receive(
        latest_snapshots: LatestSnaphots,
        latest_configurations: LatestConfigurations,
    ) -> ServerChannel {
        let (snapshot_channel, mut receiver) = tokio::sync::mpsc::channel::<ServerChange>(100);
        spawn(async move {
            loop {
                let Some(change) = receiver.recv().await else { break };
                match change {
                    ServerChange::AddClient(id) => {
                        let mut latest_snaphots = latest_snapshots.write().await;
                        latest_snaphots.order += 1;
                        latest_snaphots.clients.insert(id, Default::default());
                    }
                    ServerChange::SetSnapshot(id, snapshot) => {
                        let mut latest_snaphots = latest_snapshots.write().await;
                        if latest_snaphots.clients.contains_key(&id) {
                            latest_snaphots.order += 1;
                            latest_snaphots.clients.insert(id, snapshot);
                        }
                    }
                    ServerChange::SetConfiguration(id, config) => {
                        let mut latest_configurations = latest_configurations.write().await;
                        latest_configurations.clients.insert(id, config);
                    }
                    ServerChange::RemoveClient(id) => {
                        let mut latest_snaphots = latest_snapshots.write().await;
                        let mut latest_configurations = latest_configurations.write().await;
                        latest_snaphots.order += 1;
                        latest_snaphots.clients.remove(&id);
                        latest_configurations.clients.remove(&id);
                    }
                }
            }
        });
        snapshot_channel
    }

    pub async fn apply_snapshot(&self, id: ClientId, snapshot: LocalStateSnapshot) {
        if let Err(err) = self
            .channel
            .send(ServerChange::SetSnapshot(id, snapshot))
            .await
        {
            println!("Failed to send snapshot to be applied {:?}", err);
        }
    }

    pub async fn add_client(&self, client: Client) -> Result<()> {
        let mut clients = self.clients.lock().await;
        self.channel
            .send(ServerChange::AddClient(client.id()))
            .await?;
        clients.insert(client.id(), client);
        Ok(())
    }

    pub async fn remove_client(&self, id: ClientId) {
        let mut clients = self.clients.lock().await;
        if let Err(err) = self.channel.send(ServerChange::RemoveClient(id)).await {
            println!("Failed to remove client: {:?}", err);
        }
        clients.remove(&id);
    }

    pub async fn apply_configuration(&self, id: ClientId, config: LocalConfigurationSnapshot<()>) {
        if let Err(err) = self
            .channel
            .send(ServerChange::SetConfiguration(id, config))
            .await
        {
            println!("Failed to send configuration to be applied {:?}", err);
        }
    }

    pub async fn read_latest_snapshots(&self) -> RemoteStateSnapshot {
        self.latest_snapshots.read().await.clone()
    }
}
