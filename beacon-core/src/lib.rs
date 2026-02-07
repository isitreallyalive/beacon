//! # beacon-core

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use beacon_codec::{decode::Decode, encode::Encode};
use beacon_net::{conn::Connection, packet::RawPacket};
use bevy_ecs::prelude::*;
use flume::{Receiver, Sender};
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
        beacon_net::ecs(&mut schedule);

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
                    let (tx, rx, token) = Connection::spawn(&mut self.world);

                    // spawn a task to read packets from the socket and send them to the connection
                    tokio::spawn(read_packets(sock, addr, tx, rx, token));
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

async fn read_packets(
    sock: TcpStream,
    addr: SocketAddr,
    tx: Sender<RawPacket>,
    rx: Receiver<RawPacket>,
    token: CancellationToken,
) {
    debug!(addr = %addr, "new connection established");
    let (mut reader, mut writer) = sock.into_split();

    loop {
        tokio::select! {
            Ok(packet) = rx.recv_async() => {
                let _ = packet.encode(&mut writer).await;
            },
            res = RawPacket::decode(&mut reader) => {
                let Ok(packet) = res else { break; };
                let _ = tx.send_async(packet).await;
            }
        }
    }

    debug!(addr = %addr, "connection closed");
    token.cancel();
}
