use std::sync::Arc;

use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationDown, ConfigurationUp, DatagramDown, DatagramUp},
};
use quinn::{Connection, SendStream};
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;

use crate::dashboard::DashboardMessage;

struct InnerClient {
    connection: Connection,
    configuration_down: Mutex<SendStream>,
    ws: broadcast::Sender<DashboardMessage>,
    token: CancellationToken,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    pub(crate) async fn new(
        connection: Connection,
        ws: broadcast::Sender<DashboardMessage>,
        token: CancellationToken,
    ) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(InnerClient {
                connection: connection.clone(),
                configuration_down: Mutex::new(connection.open_uni().await?),
                ws,
                token,
            }),
        })
    }

    pub async fn handle_configuration_up(&self, message: ConfigurationUp) {
        println!("Received configuration up {:?}", message);
    }

    pub async fn handle_datagram_up(&self, message: DatagramUp) {
        println!("Received datagram up {:?}", message);
    }

    #[allow(dead_code)]
    pub async fn send_configuration(&self, message: ConfigurationDown) -> Result<()> {
        Ok(self
            .inner
            .configuration_down
            .lock()
            .await
            .write_all(&bincode::serialize(&message)?)
            .await?)
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
}
