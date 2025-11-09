//! UT3 (GameSpot) Query Protocol
//!
//! See the page on [minecraft.wiki](https://minecraft.wiki/w/Query) for protocol details.

use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use beacon_config::Config;
use beacon_java::JavaConnection;
use beacon_net::{Listener, update_listener};
use bevy_ecs::prelude::*;
use deku::{DekuContainerRead, DekuContainerWrite};

use crate::{
    packet::{CString, QueryRequest, QueryResponse, StatRequest},
    stats::{GAMETYPE, Stats},
};

#[macro_use]
extern crate tracing;

mod packet;
mod stats;
#[cfg(test)]
mod tests;

/// Token clearing interval
const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Resource)]
pub struct QueryListener {
    sock: UdpSocket,
    /// Last time challenge tokens were cleaned
    last_clear: Instant,
    /// Challenge tokens
    tokens: HashMap<SocketAddr, i32>,
}

impl Listener for QueryListener {
    fn register(world: &mut World, schedule: &mut Schedule, config: &Config) -> io::Result<()> {
        world.insert_resource(Self::new(config)?);
        schedule.add_systems((Self::update, Self::recv, Self::clear_tokens));
        Ok(())
    }

    fn new(config: &Config) -> io::Result<Self> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port))?;
        sock.set_nonblocking(true)?;
        Ok(Self {
            sock,
            last_clear: Instant::now(),
            tokens: HashMap::new(),
        })
    }

    update_listener!(QueryListener: query);
}

impl QueryListener {
    fn recv(
        query: Option<ResMut<QueryListener>>,
        config: Res<Config>,
        java_conns: Query<&JavaConnection>,
    ) -> Result<()> {
        if let Some(mut query) = query {
            let mut buf = [0u8; QueryRequest::MAX_SIZE];
            match query.sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    // deserialize packet
                    let data = &buf[..size];
                    let (_, packet) = QueryRequest::from_bytes((data, 0))?;

                    // respond
                    let online = java_conns.count() as u32;
                    let response = match packet {
                        QueryRequest::Handshake { session_id } => {
                            let challenge_token = {
                                let number = match query.tokens.get(&addr) {
                                    Some(&t) => t,
                                    None => {
                                        let t = rand::random::<i32>();
                                        query.tokens.insert(addr, t);
                                        t
                                    }
                                };
                                CString::new(&format!("{}", number))?
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
                            if query.tokens.get(&addr) != Some(&challenge_token) {
                                println!("{:?}", query.tokens.get(&addr));
                                println!("{:?}", challenge_token);
                                warn!("invalid challenge token from {}", addr);
                                return Ok(());
                            }

                            // collate stats
                            let stats = Stats {
                                motd: CString::new(&config.server.motd)?,
                                gametype: GAMETYPE.clone(),
                                map: CString::new(&config.world.name)?,
                                num_players: CString::new(&format!("{}", online))?,
                                max_players: CString::new(&format!(
                                    "{}",
                                    config.server.max_players
                                ))?,
                                host_port: config.server.port,
                                host_ip: CString::new(&config.server.ip.to_string())?,
                            };

                            QueryResponse::Stat(if full {
                                stats.full(session_id)?
                            } else {
                                stats.basic(session_id)
                            })
                        }
                    };

                    let data = response.to_bytes()?;
                    query.sock.send_to(&data, addr)?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // no data available, non-blocking
                }
                Err(e) => {
                    error!("error receiving query packet: {}", e);
                }
            }
        }
        Ok(())
    }

    fn clear_tokens(query: Option<ResMut<QueryListener>>) {
        if let Some(mut query) = query {
            let now = Instant::now();
            if now.duration_since(query.last_clear) >= CLEAR_INTERVAL {
                debug!(size = query.tokens.len(), "clearing query challenge tokens");
                query.tokens.clear();
                query.last_clear = now;
            }
        }
    }
}

impl std::ops::Deref for QueryListener {
    type Target = UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.sock
    }
}
