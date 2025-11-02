use std::{
    io,
    io::Cursor,
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use beacon_config::Config;
use beacon_net::Listener;
use bevy_ecs::prelude::*;

use crate::{
    C2S_MAGIC, HANDSHAKE, QueryListener, STAT,
    stat::{GAMETYPE, HEADER_PADDING, KV_PADDING},
};

const SESSION_ID: u32 = 0x00000000;

struct TestEnv {
    world: World,
    schedule: Schedule,
    sock: UdpSocket,
    addr: SocketAddr,
    config: Config,
}

impl TestEnv {
    /// Create a new test environment
    fn new() -> Result<Self> {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        let mut config = Config::default();
        // randomise port using time-based seed
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let port = 20000 + (seed % 10000) as u16; // range 20000-29999
        config.query.port = port;
        config.server.port = port;
        world.insert_resource(config.clone());
        QueryListener::register(&mut world, &mut schedule, &config)?;
        let addr = (config.query.ip, config.query.port).into();
        let sock = UdpSocket::bind("127.0.0.1:0")?;
        Ok(Self {
            world,
            schedule,
            sock,
            addr,
            config,
        })
    }

    /// Send a packet to the query listener
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.sock.send_to(data, self.addr)?;
        self.schedule.run(&mut self.world);
        Ok(())
    }

    /// Receive a packet from the query listener
    fn recv(&self) -> Result<Cursor<Vec<u8>>> {
        let mut buf = [0u8; 1024];
        let size = self.sock.recv(&mut buf)?;
        let mut data = Cursor::new(buf[..size].to_vec());
        data.set_position(1 + 4); // skip type + session id
        Ok(data)
    }

    /// Perform handshake and return challenge token
    fn handshake(&mut self) -> Result<i32> {
        let mut buf: Vec<u8> = vec![];
        buf.extend(&C2S_MAGIC.to_be_bytes());
        buf.push(HANDSHAKE);
        buf.extend(&SESSION_ID.to_be_bytes());
        self.send(&buf)?;
        let mut data = self.recv()?;
        let token_str = read_string(&mut data)?;
        let token: i32 = token_str.parse().unwrap_or(0);
        Ok(token)
    }
}

fn read_string<R: io::Read>(reader: &mut R) -> io::Result<String> {
    let mut buf = vec![];
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    Ok(String::from_utf8(buf).unwrap_or_default())
}

#[test]
fn test_handshake() -> Result<()> {
    let mut env = TestEnv::new()?;
    env.handshake()?;
    Ok(())
}

/// Build a basic stat packet
fn build_basic_stat(token: i32) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    buf.extend(&C2S_MAGIC.to_be_bytes());
    buf.push(STAT);
    buf.extend(&SESSION_ID.to_be_bytes());
    buf.extend(&token.to_be_bytes());
    buf
}

#[test]
fn test_basic_stat() -> Result<()> {
    let mut env = TestEnv::new()?;
    let token = env.handshake()?;
    env.send(&build_basic_stat(token))?;

    let mut data = env.recv()?;
    let config = &env.config;

    // verify expected values
    let expected = [
        config.server.motd.as_str(),
        GAMETYPE,
        config.world.name.as_str(),
        "0",
        &config.server.max_players.to_string(),
        &config.server.port.to_string(),
        &config.server.ip.to_string(),
    ];
    for &exp in expected.iter() {
        let val = read_string(&mut data)?;
        assert_eq!(val, exp);
    }

    Ok(())
}

/// Peek at the next byte in the cursor without advancing the position
fn peek(cursor: &Cursor<Vec<u8>>) -> u8 {
    let pos = cursor.position() as usize;
    cursor.get_ref().get(pos).copied().unwrap_or(0)
}

#[test]
fn test_full_stat() -> Result<()> {
    let mut env = TestEnv::new()?;
    let token = env.handshake()?;

    let mut stat_buf = build_basic_stat(token);
    stat_buf.extend(&[0u8; 4]); // indicate full stat
    env.send(&stat_buf)?;

    let mut data = env.recv()?;
    data.set_position(data.position() + HEADER_PADDING.len() as u64);
    let config = &env.config;

    // verify expected key-value pairs
    let expected = [
        ("hostname", config.server.motd.as_str()),
        ("gametype", GAMETYPE),
        ("game_id", "MINECRAFT"),
        ("version", beacon_config::VERSION),
        ("plugins", ""),
        ("map", config.world.name.as_str()),
        ("numplayers", "0"),
        ("maxplayers", &config.server.max_players.to_string()),
        ("hostport", &config.server.port.to_string()),
        ("hostip", &config.server.ip.to_string()),
    ];
    let mut i = 0;
    while peek(&data) != 0 {
        let key = read_string(&mut data)?;
        let value = read_string(&mut data)?;
        assert_eq!(key.as_str(), expected[i].0);
        assert_eq!(value.as_str(), expected[i].1);
        i += 1;
    }

    // verify expected players
    data.set_position(data.position() + KV_PADDING.len() as u64);
    let next_byte = peek(&data);
    assert_eq!(next_byte, 0); // no players

    Ok(())
}
