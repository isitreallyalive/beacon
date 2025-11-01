use std::{
    collections::HashMap,
    io::{self, Cursor},
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use beacon_config::Config;
use bevy_ecs::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};

const MAGIC: u16 = 0xFEFD;
// packet ids
const HANDSHAKE: u8 = 0x09;
const STAT: u8 = 0x00;
// hardcoded stats
const GAMETYPE: &[u8] = "SMP".as_bytes();
const GAME_ID: &[u8] = "MINECRAFT".as_bytes();

const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

fn write_string(buf: &mut Vec<u8>, s: &str) {
    buf.extend(s.as_bytes());
    buf.push(0); // null terminator
}

#[derive(Resource)]
pub struct QueryListener {
    sock: UdpSocket,
    /// Last time challenge tokens were cleaned
    last_clear: Instant,
    /// Challenge tokens
    tokens: HashMap<SocketAddr, i32>,
}

impl QueryListener {
    pub fn new(config: &Config) -> io::Result<Self> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port))?;
        sock.set_nonblocking(true)?;
        Ok(Self {
            sock,
            last_clear: Instant::now(),
            tokens: HashMap::new(),
        })
    }

    pub fn register(schedule: &mut Schedule) {
        schedule.add_systems((Self::recv, Self::clear_tokens));
    }

    fn recv(query: Option<ResMut<QueryListener>>, config: Res<Config>) -> Result<()> {
        if let Some(mut query) = query {
            let mut buf = [0u8; 1500]; // typical mtu
            match query.sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let mut data = Cursor::new(&buf[..size]);

                    // --- read the header ---
                    // validate magic number
                    let magic = data.read_u16::<BigEndian>().unwrap();
                    if magic != MAGIC {
                        return Ok(());
                    }

                    // read data
                    let packet_type = data.read_u8()?;
                    let session_id = data.read_i32::<BigEndian>()?;

                    // --- process packet ---
                    let mut out = vec![packet_type];
                    out.extend(&session_id.to_be_bytes());

                    match packet_type {
                        HANDSHAKE => {
                            // no request payload
                            // write response
                            let challenge_token: i32 = rand::random();
                            query.tokens.insert(addr, challenge_token);
                            write_string(&mut out, &challenge_token.to_string());
                        }
                        STAT => {
                            // validate token
                            let challenge_token = data.read_i32::<BigEndian>()?;
                            if query.tokens.get(&addr) != Some(&challenge_token) {
                                return Ok(());
                            }
                            // write response
                            write_string(&mut out, &config.server.motd); // motd
                            out.extend(GAMETYPE); // gametype
                            out.extend(GAME_ID); // game_id
                            out.extend(beacon_config::VERSION.as_bytes()); // version
                            write_string(&mut out, ""); // plugins
                            write_string(&mut out, &config.world.name); // map
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
