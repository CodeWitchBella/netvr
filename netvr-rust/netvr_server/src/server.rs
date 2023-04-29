use std::{
    collections::{hash_map::Entry::Occupied, HashMap},
    sync::Arc,
};

use anyhow::Result;
use netvr_data::net::{
    ClientId, ConfigurationSnapshotSet, RemoteConfigurationSnapshot, RemoteStateSnapshotSet,
    StateSnapshot,
};
use tokio::{
    spawn,
    sync::{watch, Mutex, RwLock},
};

use crate::client::Client;

#[derive(Debug)]
enum ServerChange {
    AddClient(ClientId),
    SetSnapshot(ClientId, StateSnapshot),
    SetConfiguration(ClientId, RemoteConfigurationSnapshot),
    RemoveClient(ClientId),
}

type ServerChannel = tokio::sync::mpsc::Sender<ServerChange>;
type LatestSnaphots = Arc<RwLock<RemoteStateSnapshotSet>>;
type LatestConfigurations = Arc<RwLock<watch::Sender<ConfigurationSnapshotSet>>>;

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
        let latest_configurations: LatestConfigurations =
            Arc::new(RwLock::new(watch::channel(Default::default()).0));
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
                        let latest_configurations = latest_configurations.write().await;
                        latest_snaphots.order += 1;
                        latest_snaphots.clients.insert(id, Default::default());
                        latest_configurations.send_modify(|confs| {
                            confs.clients.insert(id, Default::default());
                        });
                    }
                    ServerChange::SetSnapshot(id, snapshot) => {
                        let mut latest_snaphots = latest_snapshots.write().await;
                        if latest_snaphots.clients.contains_key(&id) {
                            latest_snaphots.order += 1;
                            latest_snaphots.clients.insert(id, snapshot);
                        }
                    }
                    ServerChange::SetConfiguration(id, config) => {
                        let latest_configurations = latest_configurations.write().await;
                        latest_configurations.send_if_modified(|confs| {
                            if let Occupied(mut e) = confs.clients.entry(id) {
                                e.insert(config);
                                true
                            } else {
                                println!("Configuration ignored {:?}", config);
                                false
                            }
                        });
                    }
                    ServerChange::RemoveClient(id) => {
                        let mut latest_snaphots = latest_snapshots.write().await;
                        let latest_configurations = latest_configurations.write().await;
                        latest_snaphots.order += 1;
                        latest_snaphots.clients.remove(&id);
                        latest_configurations.send_modify(|confs| {
                            confs.clients.remove(&id);
                        });
                    }
                }
            }
        });
        snapshot_channel
    }

    pub async fn latest_configuration(&self) -> watch::Receiver<ConfigurationSnapshotSet> {
        self.latest_configurations.read().await.subscribe()
    }

    pub async fn apply_snapshot(&self, id: ClientId, snapshot: StateSnapshot) {
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

    pub async fn get_first_client(&self) -> Option<Client> {
        self.clients
            .lock()
            .await
            .iter()
            .take(1)
            .collect::<Vec<_>>()
            .get(0)
            .map(|v| v.1)
            .cloned()
    }

    pub async fn get_clients(&self) -> Vec<(u32, Client)> {
        self.clients
            .lock()
            .await
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    #[allow(dead_code)]
    pub async fn get_client(&self, id: ClientId) -> Option<Client> {
        self.clients.lock().await.get(&id).cloned()
    }

    pub async fn remove_client(&self, id: ClientId) {
        let mut clients = self.clients.lock().await;
        if let Err(err) = self.channel.send(ServerChange::RemoveClient(id)).await {
            println!("Failed to remove client: {:?}", err);
        }
        clients.remove(&id);
    }

    pub async fn apply_configuration(&self, id: ClientId, config: RemoteConfigurationSnapshot) {
        if let Err(err) = self
            .channel
            .send(ServerChange::SetConfiguration(id, config))
            .await
        {
            println!("Failed to send configuration to be applied {:?}", err);
        }
    }

    pub async fn read_latest_snapshots(&self) -> RemoteStateSnapshotSet {
        self.latest_snapshots.read().await.clone()
    }
}
