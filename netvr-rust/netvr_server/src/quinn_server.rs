use anyhow::Result;
use quinn::{AsyncUdpSocket, Endpoint, EndpointConfig, ServerConfig, TokioRuntime};

/// Returns default server configuration along with its certificate.
fn configure_server() -> Result<(ServerConfig, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;

    Ok((server_config, cert_der))
}

/// Returns a new server endpoint and its certificate.
pub(crate) fn make_server_endpoint(socket: impl AsyncUdpSocket) -> Result<Endpoint> {
    let (server_config, _server_cert) = configure_server()?;
    let endpoint = Endpoint::new_with_abstract_socket(
        EndpointConfig::default(),
        Some(server_config),
        socket,
        TokioRuntime,
    )?;

    Ok(endpoint)
}
