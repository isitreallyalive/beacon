//! # beacon-core

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use beacon_codec::decode::Decode;
use beacon_net::{
    conn::Connection,
    packet::{RawPacket, RawPacketReceiver, RawPacketSender},
};
use bevy_ecs::prelude::*;
use miette::{IntoDiagnostic, Result};
use tokio::{
    net::{TcpListener, TcpStream},
    time::Interval,
};
use tokio_util::sync::CancellationToken;

#[macro_use]
extern crate tracing;

const TARGET_TPS: f64 = 20.;

/// The Minecraft server you'll love.
pub struct BeaconServer {
    listener: TcpListener,
    state: Arc<ServerState>,

    tick: Interval,
    world: World,
    schedule: Schedule,
    packet_tx: RawPacketSender,
}

/// Shared server state.
#[derive(Default)]
pub struct ServerState {
    cancel_token: CancellationToken,
}

impl BeaconServer {
    /// Create a new instance of beacon.
    pub async fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        // start ecs
        let tick = tokio::time::interval(Duration::from_secs_f64(1. / TARGET_TPS));
        let (mut world, mut schedule) = (World::new(), Schedule::default());
        let config = beacon_config::ecs(&mut world, &mut schedule, config_path)?;
        let packet_tx = RawPacketReceiver::ecs(&mut world, &mut schedule);

        // bind the server
        let addr: SocketAddr = (config.server.ip, config.server.port).into();
        let listener = TcpListener::bind(addr).await.into_diagnostic()?;
        info!("server listening on {}", addr);

        Ok(Self {
            listener,
            state: Arc::new(ServerState::default()),

            tick,
            world,
            schedule,
            packet_tx,
        })
    }

    /// Start the server.
    pub async fn start(&mut self) -> Result<()> {
        // listen for shutdown signals
        tokio::spawn({
            let token = self.state.cancel_token.clone();
            async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    token.cancel();
                }
            }
        });

        loop {
            tokio::select! {
                Ok((sock, addr)) = self.listener.accept() => {
                    // spawn in the ecs
                    let id = self.world.spawn(Connection::from(addr)).id();

                    // spawn a task to handle the I/O side of the connection
                    tokio::spawn(handle_connection(sock, addr, id, self.packet_tx.clone()));
                },
                _ = self.state.cancel_token.cancelled() => break,
                _ = self.tick.tick() => self.schedule.run(&mut self.world),
            }
        }

        // shutdown + cleanup
        info!("shutting down...");

        Ok(())
    }
}

async fn handle_connection(
    sock: TcpStream,
    addr: SocketAddr,
    id: Entity,
    packet_tx: RawPacketSender,
) {
    debug!(addr = %addr, "new connection established");
    let (mut reader, writer) = sock.into_split();

    loop {
        let Ok(packet) = RawPacket::decode(&mut reader).await else {
            error!(addr = %addr, "failed to read packet");
            break;
        };
        let _ = packet_tx.send((id, packet));
    }

    debug!(addr = %addr, "connection closed");
}
