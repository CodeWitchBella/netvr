use std::sync::Arc;

use tokio::{
    net::{TcpListener, UdpSocket},
    sync::{Mutex, MutexGuard},
};

struct ContextInner {
    pub server_udp: Mutex<UdpSocket>,
    pub server_tcp: Mutex<TcpListener>,
}

#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>,
}

impl Context {
    pub fn new(server_udp: UdpSocket, server_tcp: TcpListener) -> Self {
        Self {
            inner: Arc::new(ContextInner {
                server_udp: Mutex::new(server_udp),
                server_tcp: Mutex::new(server_tcp),
            }),
        }
    }
    pub async fn server_udp(&self) -> MutexGuard<'_, UdpSocket> {
        self.inner.server_udp.lock().await
    }
    pub async fn server_tcp(&self) -> MutexGuard<'_, TcpListener> {
        self.inner.server_tcp.lock().await
    }
}
