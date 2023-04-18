use std::sync::Arc;

use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationUp, DatagramDown, LocalStateSnapshot},
};
use quinn::Connection;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::dashboard::DashboardMessage;

struct InnerClient {
    id: usize,
    connection: Connection,
    ws: broadcast::Sender<DashboardMessage>,
    token: CancellationToken,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    pub(crate) fn new(
        connection: Connection,
        ws: broadcast::Sender<DashboardMessage>,
        token: CancellationToken,
    ) -> Self {
        Self {
            inner: Arc::new(InnerClient {
                id: connection.stable_id(),
                connection,
                ws,
                token,
            }),
        }
    }

    pub async fn handle_configuration_up(&self, message: ConfigurationUp) {
        println!("Received configuration up {:?}", message);
    }

    pub async fn handle_datagram_up(&self, message: LocalStateSnapshot) {
        println!("Received datagram {:?}", message);
        let _ = self.ws().send(DashboardMessage::DatagramUp {
            stable_id: self.inner.connection.stable_id(),
            message,
        });
    }

    #[allow(dead_code)]
    pub async fn send_datagram(&self, message: DatagramDown) -> Result<()> {
        let message = bincode::serialize(&message)?;
        Ok(self.inner.connection.send_datagram(message.into())?)
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
    pub(crate) fn id(&self) -> usize {
        self.inner.id
    }
}
