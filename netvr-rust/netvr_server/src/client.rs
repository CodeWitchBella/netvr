use std::sync::Arc;

use anyhow::{Ok, Result};
use netvr_data::{
    bincode,
    net::{ClientId, ConfigurationUp, StateSnapshot},
};
use quinn::Connection;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{dashboard::DashboardMessage, server::Server};

struct InnerClient {
    id: ClientId,
    ws: broadcast::Sender<DashboardMessage>,
    token: CancellationToken,
    server: Server,
}

#[derive(Clone)]
pub(crate) struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    pub(crate) fn new(
        ws: broadcast::Sender<DashboardMessage>,
        token: CancellationToken,
        server: Server,
        id: ClientId,
    ) -> Self {
        Self {
            inner: Arc::new(InnerClient {
                id,
                ws,
                token,
                server,
            }),
        }
    }

    pub async fn handle_configuration_up(&self, message: ConfigurationUp) {
        println!("Received configuration up {:?}", message);
    }

    pub async fn handle_recv_snapshot(
        &self,
        message: StateSnapshot,
        connection: &Connection,
    ) -> Result<()> {
        // println!("Received datagram {:?}", message);
        let _ = self.ws().send(DashboardMessage::DatagramUp {
            id: self.id(),
            message,
        });
        let snapshots = self.inner.server.read_latest_snapshots().await;
        // TODO: snaphots.remove(&self.id());
        // TODO: transform snaphots by Matrix
        connection.send_datagram(bincode::serialize(&snapshots)?.into())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn ws(&self) -> &broadcast::Sender<DashboardMessage> {
        &self.inner.ws
    }

    #[allow(dead_code)]
    pub(crate) fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture {
        self.inner.token.cancelled()
    }

    #[allow(dead_code)]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.inner.token.is_cancelled()
    }

    #[allow(dead_code)]
    pub(crate) fn id(&self) -> ClientId {
        self.inner.id
    }
}
