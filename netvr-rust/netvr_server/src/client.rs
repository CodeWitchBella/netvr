use std::sync::Arc;

use anyhow::{Ok, Result};
use netvr_data::{
    app, bincode,
    net::{ClientId, ConfigurationDown, ConfigurationUp, DatagramDown, StateSnapshot},
};
use quinn::Connection;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{dashboard::DashboardMessage, server::Server};

struct InnerClient {
    id: ClientId,
    ws: broadcast::Sender<DashboardMessage>,
    token: CancellationToken,
    server: Server,
    configuration_down_queue: mpsc::UnboundedSender<ConfigurationDown>,
    app_down_queue: mpsc::UnboundedSender<app::AppDown>,
    connection: Connection,
}

/// Represnets one connected client
#[derive(Clone)]
pub(crate) struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    /// Crreate a new client
    pub(crate) fn new(
        ws: broadcast::Sender<DashboardMessage>,
        token: CancellationToken,
        server: Server,
        id: ClientId,
        configuration_down_queue: mpsc::UnboundedSender<ConfigurationDown>,
        app_down_queue: mpsc::UnboundedSender<app::AppDown>,
        connection: Connection,
    ) -> Self {
        Self {
            inner: Arc::new(InnerClient {
                id,
                ws,
                token,
                server,
                configuration_down_queue,
                app_down_queue,
                connection,
            }),
        }
    }

    /// Call on configuration message
    pub async fn handle_configuration_up(&self, message: ConfigurationUp) {
        println!("Received configuration up {:?}", message);
    }

    /// Call on datagram
    pub async fn handle_recv_snapshot(&self, message: StateSnapshot) -> Result<()> {
        // println!("Received datagram {:?}", message);
        let _ = self.ws().send(DashboardMessage::DatagramUp {
            id: self.id(),
            message,
        });
        let mut snapshots = self.inner.server.read_latest_snapshots().await;
        snapshots.clients.remove(&self.id());
        self.send_datagram(&DatagramDown::State(snapshots))?;
        Ok(())
    }

    /// Dead code
    #[allow(dead_code)]
    pub(crate) fn ws(&self) -> &broadcast::Sender<DashboardMessage> {
        &self.inner.ws
    }

    /// Call when you want to send something to a client
    pub(crate) fn send_configuration_down(&self, message: ConfigurationDown) -> Result<()> {
        self.inner.configuration_down_queue.send(message)?;
        Ok(())
    }

    /// Call when you want to send something to a client
    pub(crate) fn send_app_down(&self, message: app::AppDown) -> Result<()> {
        self.inner.app_down_queue.send(message)?;
        Ok(())
    }

    /// Call when you want to send something to a client
    pub(crate) fn send_datagram(&self, datagram: &DatagramDown) -> Result<()> {
        self.inner
            .connection
            .send_datagram(bincode::serialize(&datagram)?.into())?;
        Ok(())
    }

    /// Wait until the client is cancelled
    #[allow(dead_code)]
    pub(crate) fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture {
        self.inner.token.cancelled()
    }

    /// Check if the client is cancelled
    #[allow(dead_code)]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.inner.token.is_cancelled()
    }

    /// Cancel the client
    #[allow(dead_code)]
    pub(crate) fn cancel(&self) {
        self.inner.token.cancel()
    }

    /// Get the client's id
    #[allow(dead_code)]
    pub(crate) fn id(&self) -> ClientId {
        self.inner.id
    }
}
