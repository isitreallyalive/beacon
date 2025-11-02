use std::{
    collections::HashMap,
    io::{self, Cursor},
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use beacon_config::Config;
use beacon_java::JavaConnection;
use beacon_net::{Listener, update_listener};
use bevy_ecs::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};

mod stat;

/// Magic number that every request starts with
const MAGIC: u16 = 0xFEFD;
/// Maximum size of an incoming packet (full stat)
const BUF_SIZE: usize = 2 + 1 + 4 + 4 + 4;

// packet ids
const HANDSHAKE: u8 = 0x09;
const STAT: u8 = 0x00;

//// Token clearing interval
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

pub(crate) fn write_null<W: io::Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all(&[0])?;
    Ok(())
}

pub(crate) fn write_string<W: io::Write>(writer: &mut W, s: &str) -> io::Result<()> {
    writer.write_all(s.as_bytes())?;
    write_null(writer)?;
    Ok(())
}

impl QueryListener {
    fn recv(
        query: Option<ResMut<QueryListener>>,
        config: Res<Config>,
        java_conns: Query<&JavaConnection>,
    ) -> Result<()> {
        if let Some(mut query) = query {
            let mut buf = [0u8; BUF_SIZE];
            match query.sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let mut data = Cursor::new(&buf[..size]);

                    // validate magic number
                    match data.read_u16::<BigEndian>() {
                        Ok(m) if m == MAGIC => m,
                        _ => return Ok(()),
                    };

                    // packet header
                    let packet_type = data.read_u8()?;
                    let session_id = data.read_i32::<BigEndian>()?;

                    // prepare respons
                    let mut out = vec![packet_type];
                    out.extend(&session_id.to_be_bytes());

                    match packet_type {
                        HANDSHAKE => {
                            // no request payload
                            // write response
                            let challenge_token: i32 = rand::random();
                            query.tokens.insert(addr, challenge_token);
                            write_string(&mut out, &challenge_token.to_string())?;
                        }
                        STAT => {
                            // validate token
                            let challenge_token = data.read_i32::<BigEndian>()?;
                            if query.tokens.get(&addr) != Some(&challenge_token) {
                                return Ok(());
                            }
                            // build stats
                            let stats = stat::Stats {
                                motd: &config.server.motd,
                                map: &config.world.name,
                                numplayers: &java_conns.iter().count().to_string(),
                                maxplayers: &config.server.max_players.to_string(),
                                hostport: &config.server.port.to_string(),
                                hostip: &config.server.ip.to_string(),
                            };
                            // is the payload exactly 4 bytes (challenge token)?
                            let remaining = size.saturating_sub(data.position() as usize);
                            if remaining == 4 {
                                stats.full(&mut out)?;
                            } else {
                                stats.basic(&mut out)?;
                            }
                        }
                        _ => return Ok(()),
                    }

                    query.sock.send_to(&out, addr)?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // no data available, non-blocking
                }
                Err(e) => {
                    eprintln!("Error receiving query packet: {}", e);
                }
            }
        }
        Ok(())
    }

    fn clear_tokens(query: Option<ResMut<QueryListener>>) {
        if let Some(mut query) = query {
            let now = Instant::now();
            if now.duration_since(query.last_clear) >= CLEAR_INTERVAL {
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
