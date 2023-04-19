use netvr_data::bincode;
use tokio::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("netvr connection encountered IO problem")]
    IO(#[from] io::Error),

    #[error("QUIC connection error")]
    Connection(#[from] quinn::ConnectionError),

    #[error("Bincode encode/decode error")]
    Bincode(#[from] bincode::Error),

    #[error("QUIC write error")]
    WriteError(#[from] quinn::WriteError),

    #[error("Framing Error")]
    FramingError(#[from] netvr_data::FramingError),

    #[error("unknown netvr connect error")]
    Unknown,
}
