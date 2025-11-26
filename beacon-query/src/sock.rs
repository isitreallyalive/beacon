use std::{
    net::SocketAddr,
    ops,
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use deku::DekuContainerRead;
use futures::Stream;
use kameo::{message::StreamMessage, prelude::*};
use tokio::{io::ReadBuf, net, task::JoinHandle};

use crate::{
    QueryActor,
    req::{MAX_SIZE, QueryRequest},
};

type UdpPacket = (QueryRequest, SocketAddr);
pub type UdpMessage = StreamMessage<UdpPacket, (), ()>;

/// UDP socket that manages its own stream handler.
pub struct UdpSocket {
    sock: Arc<net::UdpSocket>,
    handle: JoinHandle<Result<UdpStream, SendError<UdpMessage>>>,
}

impl UdpSocket {
    pub async fn bind(
        addr: impl Into<SocketAddr>,
        actor_ref: &ActorRef<QueryActor>,
    ) -> std::io::Result<Self> {
        // bind the udp socket and register the stream handler
        let sock = Arc::new(net::UdpSocket::bind(addr.into()).await?);
        let handle = actor_ref.attach_stream(UdpStream(sock.clone()), (), ());
        Ok(Self { sock, handle })
    }
}

impl ops::Deref for UdpSocket {
    type Target = net::UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.sock
    }
}

impl ops::Drop for UdpSocket {
    fn drop(&mut self) {
        // kill the old stream handler
        self.handle.abort();
    }
}

/// UDP stream that yields [QueryRequest] as they arrive.
struct UdpStream(Arc<net::UdpSocket>);

impl Stream for UdpStream {
    type Item = UdpPacket;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = vec![0u8; MAX_SIZE];
        let mut buf = ReadBuf::new(&mut buf);

        match self.0.poll_recv_from(ctx, &mut buf) {
            Poll::Ready(Ok(addr)) => {
                let Ok((_, req)) = QueryRequest::from_bytes((buf.filled(), 0)) else {
                    return Poll::Ready(None);
                };
                Poll::Ready(Some((req, addr)))
            }
            Poll::Ready(Err(_)) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
