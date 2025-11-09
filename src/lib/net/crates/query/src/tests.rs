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

const SESSION_ID: i32 = 0x12345678;

struct TestEnv {
    world: World,
    schedule: Schedule,
    server_addr: SocketAddr,
    client_sock: UdpSocket,
    config: Config,
}

impl TestEnv {
    fn new() -> io::Result<Self> {
        let (mut world, mut schedule) = (World::new(), Schedule::default());
        let mut config = Config::default();
        config.query.port = rand::random();
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

    fn recv<P: for<'a> DekuContainerRead<'a>>(&mut self) -> io::Result<P> {
        let mut buf = [0u8; 1024];
        let size = self.client_sock.recv(&mut buf)?;
        let (_, packet) = P::from_bytes((&buf[..size], 0))?;
        Ok(packet)
    }

    fn send<P: DekuContainerWrite>(&mut self, packet: P) -> io::Result<()> {
        let data = packet.to_bytes()?;
        self.client_sock.send_to(&data, self.server_addr)?;
        self.schedule.run(&mut self.world);
        Ok(())
    }

    fn handshake(&mut self) -> Result<i32> {
        self.send(QueryRequest::Handshake {
            session_id: SESSION_ID,
        })?;
        let response = self.recv::<QueryResponse>()?;
        match response {
            QueryResponse::Handshake {
                challenge_token, ..
            } => Ok(challenge_token.0.to_string_lossy().parse()?),
            _ => Err("unexpected response type")?,
        }
    }

    fn stat_request(&mut self, full: bool) -> Result<StatResponseKind> {
        let challenge_token = self.handshake()?;
        self.send(QueryRequest::Stat(StatRequest {
            session_id: SESSION_ID,
            challenge_token,
            full,
        }))?;
        let response = self.recv::<QueryResponse>()?;
        match response {
            QueryResponse::Stat(kind) => Ok(kind),
            _ => Err("unexpected response type")?,
        }
    }
}

#[test]
fn test_handshake() -> Result<()> {
    TestEnv::new()?.handshake()?;
    Ok(())
}

#[test]
fn test_basic_stat() -> Result<()> {
    let mut env = TestEnv::new()?;
    let response = env.stat_request(false)?;

    match response {
        StatResponseKind::Basic { basic, .. } => {
            assert_eq!(basic.motd.to_string_lossy(), env.config.server.motd);
            assert_eq!(basic.map.to_string_lossy(), env.config.world.name);
            assert_eq!(basic.num_players.to_string_lossy(), "0");
            assert_eq!(
                basic.max_players.to_string_lossy(),
                env.config.server.max_players.to_string()
            );
            assert_eq!(basic.host_port, env.config.server.port);
            assert_eq!(
                basic.host_ip.to_string_lossy(),
                env.config.server.ip.to_string()
            );
        }
        _ => Err("expected basic stat response")?,
    }
    Ok(())
}

#[test]
fn test_full_stat() -> Result<()> {
    let mut env = TestEnv::new()?;
    let response = env.stat_request(true)?;

    match response {
        StatResponseKind::Full { kv, .. } => {
            let expected = [
                ("hostname", &env.config.server.motd),
                ("map", &env.config.world.name),
                ("numplayers", &"0".to_string()),
                ("maxplayers", &env.config.server.max_players.to_string()),
                ("hostport", &env.config.server.port.to_string()),
                ("hostip", &env.config.server.ip.to_string()),
            ];

            for (key, value) in expected {
                let actual = kv.get(&CString::new(key)?).ok_or("missing key in kv")?;
                assert_eq!(actual.to_string_lossy(), *value);
            }
        }
        _ => Err("expected full stat response")?,
    }
    Ok(())
}
