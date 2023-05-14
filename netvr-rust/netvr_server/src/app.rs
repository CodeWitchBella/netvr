use anyhow::Result;
use netvr_data::{
    app::{AppDatagramUp, AppDown, AppUp, Snapshot},
    net::{ClientId, DatagramDown::App},
    Pose,
};
use tokio::{select, sync::mpsc, time::Interval};

use crate::server::Server;

#[derive(Clone, Debug)]
struct AppObject {
    owner: ClientId,
    pose: Pose,
}

pub(crate) struct AppServer {
    channel: mpsc::UnboundedReceiver<AppServerMessage>,
    initial_state: Vec<AppObject>,
    state: Vec<AppObject>,
    server: Server,
}

enum UpMessage {
    IntervalLapsed,
    SetPose(ClientId, usize, Pose),
    Init(Snapshot),
    Grab(ClientId, u32),
    ResetObjects,
}

#[derive(Debug)]
pub(crate) enum AppServerMessage {
    Datagram(ClientId, AppDatagramUp),
    AppUp(ClientId, AppUp),
    ResetObjects,
}

pub(crate) type AppChannel = mpsc::UnboundedSender<AppServerMessage>;

impl AppServer {
    pub(crate) fn start(server: Server) -> (Self, AppChannel) {
        let channel = mpsc::unbounded_channel::<AppServerMessage>();

        (
            Self {
                channel: channel.1,
                initial_state: Default::default(),
                state: Default::default(),
                server,
            },
            channel.0,
        )
    }

    async fn recv_flat(&mut self, interval: &mut Interval) -> Result<UpMessage> {
        Ok(select!(
            message = self.channel.recv() => match message {
                Some(data) => match data {
                    AppServerMessage::Datagram(client_id, datagram) => match datagram {
                        AppDatagramUp::SetPose(object_id, pose) => {
                            UpMessage::SetPose(client_id, object_id, pose)
                        }
                    },
                    AppServerMessage::AppUp(client_id, message) => match message {
                        AppUp::Init(snaphot) => UpMessage::Init(snaphot),
                        AppUp::Grab(object_id) => UpMessage::Grab(client_id, object_id),
                    },
                    AppServerMessage::ResetObjects => {
                        UpMessage::ResetObjects
                    }
                },
                None => Err(anyhow::anyhow!("AppServer channel closed"))?,
            },
            _ = interval.tick() => UpMessage::IntervalLapsed,
        ))
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(20));
        loop {
            match self.recv_flat(&mut interval).await? {
                UpMessage::SetPose(client_id, object_id, pose) => {
                    if let Some(mut entry) = self.state.get_mut(object_id) {
                        if entry.owner == client_id {
                            entry.pose = pose;
                        } else {
                            println!(
                                "Received pose update for object {} from unauthorized client {}. \
                                 This is probably okay.",
                                object_id, client_id
                            );
                        }
                    } else {
                        println!("Received pose update for unknown object {}", object_id);
                    }
                }
                UpMessage::Init(snapshot) => {
                    if !self.initial_state.is_empty() {
                        continue;
                    }
                    for pose in snapshot.objects.iter() {
                        self.initial_state.push(AppObject {
                            owner: 0,
                            pose: pose.to_owned(),
                        });
                    }
                    self.state = self.initial_state.clone();
                }
                UpMessage::Grab(client_id, object_id) => {
                    if let Some(ref mut entry) = self.state.get_mut(object_id as usize) {
                        if let Some(client) = self.server.get_client(entry.owner).await {
                            if let Err(e) = client.send_app_down(AppDown::Release(object_id)) {
                                println!("Failed to send release to client: {}", e);
                            }
                        }
                        entry.owner = client_id;
                    } else {
                        println!("Received grab for unknown object {}", object_id);
                    }
                }
                UpMessage::IntervalLapsed => {
                    let mut snapshot = Snapshot::default();
                    for object in self.state.iter() {
                        snapshot.objects.push(object.pose.to_owned());
                    }
                    let clients = self.server.get_clients().await;
                    let message = App(snapshot.clone());
                    for (client_id, client) in clients.iter() {
                        if let Err(e) = client.send_datagram(&message) {
                            println!("Failed to send app datagram to client {}: {}", client_id, e)
                        }
                    }
                }
                UpMessage::ResetObjects => {
                    self.state = self.initial_state.clone();
                }
            };
        }
    }
}
