use quinn::AsyncUdpSocket;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{self, Interest};

/// Implementation of `quinn::AsyncUdpSocket` for `tokio::net::UdpSocket`, but
/// using Arc instead of direct ownership.
#[derive(Debug)]
pub(crate) struct MySocket {
    io: Arc<tokio::net::UdpSocket>,
    inner: quinn_udp::UdpSocketState,
}

impl MySocket {
    pub fn new(io: Arc<tokio::net::UdpSocket>) -> Self {
        Self {
            io,
            inner: quinn_udp::UdpSocketState::default(),
        }
    }
}

// The rest of this file is adapted from
// https://github.com/quinn-rs/quinn/blob/0.9.3/quinn/src/runtime/tokio.rs
macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}
impl AsyncUdpSocket for MySocket {
    fn poll_send(
        &mut self,
        state: &quinn_udp::UdpState,
        cx: &mut Context,
        transmits: &[quinn_proto::Transmit],
    ) -> Poll<io::Result<usize>> {
        let inner = &mut self.inner;
        let io = &self.io;
        loop {
            ready!(io.poll_send_ready(cx))?;
            if let Ok(res) = io.try_io(Interest::WRITABLE, || {
                inner.send(io.as_ref().into(), state, transmits)
            }) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [std::io::IoSliceMut<'_>],
        meta: &mut [quinn_udp::RecvMeta],
    ) -> Poll<io::Result<usize>> {
        loop {
            ready!(self.io.poll_recv_ready(cx))?;
            if let Ok(res) = self.io.try_io(Interest::READABLE, || {
                self.inner.recv(self.io.as_ref().into(), bufs, meta)
            }) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.io.local_addr()
    }
}
