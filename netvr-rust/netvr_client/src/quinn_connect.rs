use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use quinn::{ClientConfig, Connection, Endpoint};

use crate::error::Error;

/// Connects the the server and returns the endpoint and connection.
/// Makes sure that certificates are ignored.
pub(crate) async fn quinn_connect(
    server_addr: SocketAddr,
) -> Result<(Endpoint, Connection), Error> {
    let client_cfg = configure_client();
    let mut endpoint = Endpoint::client(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0))?;
    endpoint.set_default_client_config(client_cfg);

    // connect to server
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();

    Ok((endpoint, connection))
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for
/// testing.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
