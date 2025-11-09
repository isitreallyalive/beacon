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
    packet::{QueryRequest, QueryResponse},
};

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
            QueryResponse::Handshake { token } => {
                let number: i32 = token.0.to_string_lossy().parse()?;
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
    beacon.send(QueryRequest::BasicStat)?;
    Ok(())
}

#[test]
fn test_full_stat() -> Result<()> {
    let mut beacon = TestEnv::new()?;
    beacon.send(QueryRequest::FullStat)?;
    Ok(())
}
