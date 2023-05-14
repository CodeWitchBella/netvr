#![cfg(feature = "nonweb")]

use quinn::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FramingError {
    #[error("failed to read from stream")]
    ReadExactError(#[from] quinn::ReadExactError),
    #[error("failed to write to stream")]
    WriteError(#[from] quinn::WriteError),
    #[error("failed to deserialize")]
    DecodeError(#[from] bincode::Error),
    #[error("Received invalid setup frame")]
    InvalidSetupFrame([u8; 8], [u8; 8]),
    #[error("Timed out waiting for setup frame")]
    SetupTimeout,
    #[error("Connection error")]
    ConnectionError(#[from] quinn::ConnectionError),
}

pub struct RecvFrames<T: serde::de::DeserializeOwned> {
    __marker: std::marker::PhantomData<T>,
    inner: quinn::RecvStream,
    buffer: Vec<u8>,
}

impl<T: serde::de::DeserializeOwned> RecvFrames<T> {
    pub async fn open(connection: &Connection, id: &[u8; 8]) -> Result<Self, FramingError> {
        tokio::select!(
            res = Self::open_inner(connection, id) => res,
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => Err(FramingError::SetupTimeout),
        )
    }

    async fn open_inner(connection: &Connection, id: &[u8; 8]) -> Result<Self, FramingError> {
        let mut con = connection.accept_uni().await?;
        let mut buf = [0u8; 8];
        con.read_exact(&mut buf).await?;
        if id != &buf {
            return Err(FramingError::InvalidSetupFrame(buf, *id));
        }
        Ok(Self {
            __marker: std::marker::PhantomData,
            inner: con,
            buffer: Vec::new(),
        })
    }

    // todo rename to recv
    pub async fn read(&mut self) -> Result<T, FramingError> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf).await?;
        let len = usize::from_le_bytes(buf);
        self.buffer.resize(len, 0);
        self.inner.read_exact(&mut self.buffer).await?;
        Ok(bincode::deserialize::<T>(&self.buffer)?)
    }
}

pub struct SendFrames<T: serde::ser::Serialize> {
    __marker: std::marker::PhantomData<T>,
    inner: quinn::SendStream,
}

impl<T: serde::ser::Serialize> SendFrames<T> {
    pub async fn open(connection: &Connection, id: &[u8; 8]) -> Result<Self, FramingError> {
        tokio::select!(
            res = Self::open_inner(connection, id) => res,
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => Err(FramingError::SetupTimeout),
        )
    }

    async fn open_inner(connection: &Connection, id: &[u8; 8]) -> Result<Self, FramingError> {
        let mut con = connection.open_uni().await?;
        con.write_all(id).await?;
        Ok(Self {
            __marker: std::marker::PhantomData,
            inner: con,
        })
    }

    pub async fn write(&mut self, value: &T) -> Result<(), FramingError> {
        let data = bincode::serialize::<T>(value)?;
        self.inner.write_all(&data.len().to_le_bytes()).await?;
        self.inner.write_all(&data).await?;
        Ok(())
    }
}
