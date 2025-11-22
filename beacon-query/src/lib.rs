//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::{collections::HashMap, ffi::CString, io, net::SocketAddr, time::{Duration, Instant}};

use deku::{DekuContainerRead, DekuContainerWrite};
use tokio::net::UdpSocket;

use crate::{req::QueryRequest, res::QueryResponse};

#[macro_use]
extern crate tracing;

mod req;
mod res;
#[cfg(test)]
mod tests;

const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

pub struct QueryHandler {
    sock: UdpSocket,
    /// Challenge tokens mapped by client address.
    tokens: HashMap<SocketAddr, i32>,
    /// Last time tokens were cleared.
    last_cleared: Instant,
}

impl QueryHandler {
    pub async fn new() -> io::Result<Self> {
        Ok(Self {
            sock: UdpSocket::bind("0.0.0.0:25565").await?,
            tokens: HashMap::new(),
            last_cleared: Instant::now(),
        })
    }

    pub async fn tick(&mut self) -> io::Result<()> {
        // clear tokens periodically
        let elapsed = self.last_cleared.elapsed();
        if elapsed >= CLEAR_INTERVAL && !self.tokens.is_empty() {
            self.tokens.clear();
            self.last_cleared = Instant::now();
            debug!("clearing challenge tokens");
        }

        // read a packet
        let mut buf = [0u8; req::MAX_SIZE];
        let (len, addr) = self.sock.recv_from(&mut buf).await?;
        let (_, packet) = QueryRequest::from_bytes((&buf[..len], 0))?;
        info!("received {:?} from {}", packet, addr);

        // respond
        let res = self.handle(packet, addr)?;
        self.sock.send_to(&res.to_bytes()?, addr).await?;

        Ok(())
    }

    fn handle(&mut self, req: QueryRequest, addr: SocketAddr) -> io::Result<QueryResponse> {
        Ok(match req {
            QueryRequest::Handshake { session_id } => {
                // generate and store a challenge token
                let challenge_token = {
                    let value: i32 = rand::random();
                    self.tokens.insert(addr, value);
                    CString::new(value.to_string())?
                };

                QueryResponse::Handshake {
                    session_id,
                    challenge_token,
                }
            }
            _ => unimplemented!(),
        })
    }
}
