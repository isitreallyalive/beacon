//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::{
    cell::LazyCell,
    collections::HashMap,
    ffi::CString,
    io,
    net::SocketAddr,
    time::{Duration, Instant},
};

use deku::{DekuContainerRead, DekuContainerWrite};
use tokio::net::UdpSocket;

use crate::{
    req::{QueryRequest, StatRequest},
    res::QueryResponse,
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

macro_rules! lazy_string {
    ($(
        $name:ident = $value:expr $(;)? // optional trailing semicolon
    );+) => {
        $(
            const $name: LazyCell<CString> = LazyCell::new(|| CString::new($value).unwrap());
        )+
    };
}

lazy_string! {
    // full stat keys
    HOSTNAME_KEY = "hostname";
    GAMETYPE_KEY = "gametype";
    GAME_ID_KEY = "game_id";
    VERSION_KEY = "version";
    PLUGINS_KEY = "plugins";
    MAP_KEY = "map";
    NUMPLAYERS_KEY = "numplayers";
    MAXPLAYERS_KEY = "maxplayers";
    HOSTPORT_KEY = "hostport";
    HOSTIP_KEY = "hostip";

    // hard-coded full stat values
    GAME_TYPE = "SMP";
    GAME_ID = "MINECRAFT";
    VERSION = beacon_data::SUPPORTED_VERSION;
    PLUGINS = ""; // no plugins
}

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
        debug!("received {:?} from {}", packet, addr);

        // respond
        let res = self.handle(packet, addr)?;
        self.sock.send_to(&res.to_bytes()?, addr).await?;

        Ok(())
    }

    fn handle(&mut self, req: QueryRequest, addr: SocketAddr) -> io::Result<QueryResponse> {
        // todo: get this data from the server
        let motd = CString::new("Beacon Server")?;
        let map = CString::new("world")?;
        let num_players = CString::new("0")?;
        let max_players = CString::new("20")?;
        let host_port = 25565;
        let host_ip = CString::new("127.0.0.1")?;

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
