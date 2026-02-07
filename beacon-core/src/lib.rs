//! # beacon-core

use std::{net::SocketAddr, path::Path, sync::Arc};

use beacon_codec::{ProtocolState, decode::Decode, encode::Encode};
use beacon_config::Config;
use beacon_net::{conn::Connection, packet::RawPacket};
use bevy_ecs::prelude::*;
use miette::{IntoDiagnostic, Result};
use peekable::tokio::AsyncPeekable;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

#[macro_use]
extern crate tracing;

mod legacy;

/// The Minecraft server you'll love.
pub struct BeaconServer {
    listener: TcpListener,
    state: Arc<ServerState>,

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
                Ok((sock, addr)) = self.listener.accept() => spawn_connection(sock, addr, &mut self.world).await,
                _ = self.state.cancel_token.cancelled() => break,
                _ = tokio::task::yield_now() => self.schedule.run(&mut self.world),
            }
        }

        // shutdown + cleanup
        info!("shutting down...");

        Ok(())
    }
}

async fn spawn_connection(sock: TcpStream, addr: SocketAddr, world: &mut World) {
    // split socket
    let (reader, mut writer) = sock.into_split();
    let mut reader = AsyncPeekable::new(reader);

    // check for legacy server list ping
    let mut buf = [0; 2];
    let _ = reader.peek(&mut buf).await;

    // todo: detect beta 1.8-1.4 joins
    if buf[0] == 0xFE {
        let config = world.resource::<Config>();
        if !config.server.status {
            return;
        }

        let v2 = buf[1] == 0x01; // 1.4-1.6
        let motd = config.server.motd.clone();
        let max_players = config.server.max_players;
        // todo: query for real, online players
        let online = world.query::<&ProtocolState>().iter(world).count() as u32;
        legacy::handle(v2, writer, &motd, online, max_players).await;

        return;
    }

    // spawn in the ecs
    let (tx, rx, despawn) = Connection::spawn(world);

    // spawn a task to read packets from the socket and send them to the connection
    tokio::spawn(async move {
        debug!(addr = %addr, "new connection established");
        loop {
            tokio::select! {
                Ok(packet) = rx.recv_async() => {
                    let _ = packet.encode(&mut writer).await;
                },
                res = RawPacket::decode(&mut reader) => {
                    let Ok(packet) = res else { break; };
                    let _ = tx.send_async(packet).await;
                },
                _ = despawn.cancelled() => break,
            }
        }

        debug!(addr = %addr, "connection closed");
        if !despawn.is_cancelled() {
            despawn.cancel()
        };
    });
}
