//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::{
    collections::HashMap,
    ffi::CString,
    io,
    net::SocketAddr,
    rc::Rc,
    time::{Duration, Instant},
};

use beacon_config::Config;
use deku::{DekuContainerRead, DekuContainerWrite};
use tokio::{net::UdpSocket, sync::Mutex};

use crate::{
    req::{QueryRequest, StatRequest},
    res::*,
};

#[macro_use]
extern crate tracing;

mod kv;
mod req;
mod res;
#[cfg(test)]
mod tests;

/// How often to clear challenge tokens.
const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

pub struct QueryHandler {
    sock: UdpSocket,
    config: Rc<Mutex<Config>>,
    /// Challenge tokens mapped by client address.
    tokens: HashMap<SocketAddr, i32>,
    /// Last time tokens were cleared.
    last_cleared: Instant,
}

impl QueryHandler {
    pub async fn new(config: Rc<Mutex<Config>>) -> io::Result<Self> {
        let lock = config.lock().await;
        let sock = UdpSocket::bind((lock.query.ip, lock.query.port)).await?;
        drop(lock);

        Ok(Self {
            sock,
            config,
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
        debug!("received {:?} from {}", packet, addr);

        // respond
        let res = self.handle(packet, addr).await?;
        self.sock.send_to(&res.to_bytes()?, addr).await?;

        Ok(())
    }

    async fn handle(&mut self, req: QueryRequest, addr: SocketAddr) -> io::Result<QueryResponse> {
        let config = self.config.lock().await;
        let motd = CString::new(config.server.motd.clone())?;
        let map = CString::new(config.world.name.clone())?;
        let num_players = CString::new("0")?; // todo: get from server somehow
        let max_players = CString::new(config.server.max_players.to_string())?;
        let host_port = config.server.port;
        let host_ip = CString::new(config.server.ip.to_string())?;

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
            QueryRequest::Stat(StatRequest {
                session_id,
                challenge_token,
                full,
            }) => {
                // validate challenge token
                let expected_token = self.tokens.get(&addr);
                if expected_token != Some(&challenge_token) {
                    warn!(
                        expected = expected_token,
                        got = challenge_token,
                        "invalid challenge token from {}",
                        addr
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "invalid challenge token",
                    ));
                }

                if full {
                    QueryResponse::FullStat {
                        session_id,
                        kv_marker: res::KV_MARKER,
                        kv: {
                            let mut kv = kv::KeyValue::default();
                            kv.insert(HOSTNAME_KEY.clone(), motd);
                            kv.insert(GAMETYPE_KEY.clone(), GAME_TYPE.clone());
                            kv.insert(GAME_ID_KEY.clone(), GAME_ID.clone());
                            kv.insert(VERSION_KEY.clone(), VERSION.clone());
                            kv.insert(PLUGINS_KEY.clone(), PLUGINS.clone());
                            kv.insert(MAP_KEY.clone(), map);
                            kv.insert(NUMPLAYERS_KEY.clone(), num_players);
                            kv.insert(MAXPLAYERS_KEY.clone(), max_players);
                            kv.insert(HOSTPORT_KEY.clone(), CString::new(host_port.to_string())?);
                            kv.insert(HOSTIP_KEY.clone(), host_ip);
                            kv
                        },
                        player_marker: res::PLAYER_MARKER,
                        // todo: fetch player list from server
                        players: vec![],
                        nul: 0,
                    }
                } else {
                    QueryResponse::BasicStat {
                        session_id,
                        motd,
                        game_type: GAME_TYPE.clone(),
                        map,
                        num_players,
                        max_players,
                        host_port,
                        host_ip,
                    }
                }
            }
        })
    }
}
