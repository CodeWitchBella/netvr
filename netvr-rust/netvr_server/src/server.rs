use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use netvr_data::net::UdpDatagramUp;
use tokio::sync::Mutex;

struct InnerClient {
    addr: SocketAddr,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<InnerClient>>,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerClient { addr })),
        }
    }

    pub async fn handle_udp(self, buf: UdpDatagramUp) {
        todo!()
    }
}
