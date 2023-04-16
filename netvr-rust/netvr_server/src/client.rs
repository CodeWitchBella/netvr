use anyhow::Result;
use netvr_data::{
    bincode,
    net::{ConfigurationDown, ConfigurationUp, DatagramDown, DatagramUp},
};
use quinn::{Connection, SendStream};
use std::sync::Arc;
use tokio::sync::Mutex;

struct InnerClient {
    connection: Connection,
    configuration_down: Mutex<SendStream>,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    pub async fn new(connection: Connection) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(InnerClient {
                connection: connection.clone(),
                configuration_down: Mutex::new(connection.open_uni().await?),
            }),
        })
    }

    pub async fn handle_configuration_up(&self, message: ConfigurationUp) {}
    pub async fn handle_datagram_up(&self, message: DatagramUp) {}

    pub async fn send_configuration(&self, message: ConfigurationDown) -> Result<()> {
        Ok(self
            .inner
            .configuration_down
            .lock()
            .await
            .write_all(&bincode::serialize(&message)?)
            .await?)
    }

    pub async fn send_datagram(&self, message: DatagramDown) -> Result<()> {
        let message = bincode::serialize(&message)?;
        Ok(self.inner.connection.send_datagram(message.into())?)
    }
}
