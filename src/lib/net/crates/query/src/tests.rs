use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use beacon_config::Config;
use beacon_net::Listener;
use bevy_ecs::prelude::*;
use deku::{DekuContainerRead, DekuContainerWrite};

use crate::{
    QueryListener,
    packet::{CString, QueryRequest, QueryResponse, StatRequest, StatResponseKind},
};

const SESSION_ID: i32 = 0;

struct TestEnv {
    world: World,
    schedule: Schedule,
    server_addr: SocketAddr,
    client_sock: UdpSocket,
    config: Config,
}

impl TestEnv {
    /// Create a new test environment
    fn new() -> io::Result<Self> {
        // setup bevy
        let (mut world, mut schedule) = (World::new(), Schedule::default());
        let mut config = Config::default();
        config.query.port = rand::random(); // randomise port to avoid conflicts
        world.insert_resource(config.clone());
        QueryListener::register(&mut world, &mut schedule, &config)?;

        let server_addr = (config.query.ip, config.query.port).into();
        let client_sock = UdpSocket::bind("127.0.0.1:0")?;

        Ok(Self {
            world,
            schedule,
            server_addr,
            client_sock,
            config,
        })
    }

    /// Receive a packet from the query listener
    fn recv<P: for<'a> DekuContainerRead<'a>>(&mut self) -> io::Result<P> {
        let mut buf = [0u8; 1024]; // 1024 should be more than enough
        let size = self.client_sock.recv(&mut buf)?;
        let owned_buf = buf[..size].to_vec();
        let (_, packet) = P::from_bytes((&owned_buf[..], 0))?;
        Ok(packet)
    }

    /// Send a packet to the query listener
    fn send<P: DekuContainerWrite>(&mut self, packet: P) -> io::Result<()> {
        let data = packet.to_bytes()?; // serialize packet
        self.client_sock.send_to(&data, self.server_addr)?; // send packet to server
        self.schedule.run(&mut self.world); // process systems
        Ok(())
    }

    /// Handshake with the query listener. Returns the challenge token.
    fn handshake(&mut self) -> Result<i32> {
        self.send(QueryRequest::Handshake)?;
        let response = self.recv::<QueryResponse>()?;
        match response {
            QueryResponse::Handshake { challenge_token } => {
                let number: i32 = challenge_token.0.to_string_lossy().parse()?;
                Ok(number)
            }
            _ => Err("unexpected response type")?,
        }
    }
}

#[test]
fn test_handshake() -> Result<()> {
    let mut beacon = TestEnv::new()?;
    beacon.handshake()?;
    Ok(())
}

#[test]
fn test_basic_stat() -> Result<()> {
    let mut beacon = TestEnv::new()?;
    let challenge_token = beacon.handshake()?;
    beacon.send(QueryRequest::Stat(StatRequest {
        session_id: SESSION_ID,
        challenge_token,
        full: false,
    }))?;
    let response = beacon.recv::<QueryResponse>()?;

    match response {
        QueryResponse::Stat(StatResponseKind::Basic { basic, .. }) => {
            assert_eq!(basic.motd.to_string_lossy(), beacon.config.server.motd);
            assert_eq!(basic.map.to_string_lossy(), beacon.config.world.name);
            assert_eq!(basic.num_players.to_string_lossy(), "0");
            assert_eq!(
                basic.max_players.to_string_lossy(),
                format!("{}", beacon.config.server.max_players)
            );
            assert_eq!(basic.host_port, beacon.config.server.port);
            assert_eq!(
                basic.host_ip.to_string_lossy(),
                beacon.config.server.ip.to_string()
            );
        }
        _ => Err("unexpected response type")?,
    }
    Ok(())
}

#[test]
fn test_full_stat() -> Result<()> {
    let mut beacon = TestEnv::new()?;
    let challenge_token = beacon.handshake()?;
    beacon.send(QueryRequest::Stat(StatRequest {
        session_id: SESSION_ID,
        challenge_token,
        full: true,
    }))?;
    let response = beacon.recv::<QueryResponse>()?;

    match response {
        QueryResponse::Stat(StatResponseKind::Full { kv, .. }) => {
            let expected = [
                ("hostname", beacon.config.server.motd),
                ("map", beacon.config.world.name),
                ("numplayers", "0".to_string()),
                (
                    "maxplayers",
                    format!("{}", beacon.config.server.max_players),
                ),
                ("hostport", format!("{}", beacon.config.server.port)),
                ("hostip", beacon.config.server.ip.to_string()),
            ];

            for (key, value) in expected {
                let actual = kv.get(&CString::new(key)?).ok_or("missing key in kv")?;
                assert_eq!(actual.to_string_lossy(), value);
            }
        }
        _ => Err("unexpected response type")?,
    }

    Ok(())
}
