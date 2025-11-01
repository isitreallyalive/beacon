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
const HANDSHAKE: u8 = 0x09;
const STAT: u8 = 0x00;
const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

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

    fn recv(query: Option<ResMut<QueryListener>>) -> Result<()> {
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

                    println!("Received query packet type {} from {}", packet_type, addr);

                    match packet_type {
                        HANDSHAKE => {
                            // no payload
                            let challenge_token: i32 = rand::random();
                            query.tokens.insert(addr, challenge_token);
                            // turn into null terminated string
                            let token_str = format!("{}\0", challenge_token);
                            out.extend(token_str.as_bytes());
                            query.sock.send_to(&out, addr)?;
                        }
                        STAT => {}
                        _ => return Ok(()),
                    }
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
