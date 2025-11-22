use std::io;

use deku::DekuContainerRead;

use crate::req::QueryRequest;

#[macro_use]
extern crate tracing;

mod req;

pub struct QueryHandler {
    sock: tokio::net::UdpSocket
}

impl QueryHandler {
    pub async fn new() -> io::Result<Self> {
        let sock = tokio::net::UdpSocket::bind("0.0.0.0:25565").await?;
        Ok(Self { sock })
    }

    pub async fn tick(&self) -> io::Result<()> {
        // read packet
        let mut buf = [0u8; req::MAX_SIZE];
        let (len, addr) = self.sock.recv_from(&mut buf).await?;
        let (_, packet) = QueryRequest::from_bytes((&buf[..len], 0))?;
        info!("received {:?} from {}", packet, addr);

        Ok(())
    }
}