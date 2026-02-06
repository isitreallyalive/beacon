//! # beacon-core

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use beacon_codec::decode::Decode;
use beacon_config::Config;
use beacon_net::RawPacket;
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
        let config = beacon_config::register(&mut world, &mut schedule, config_path)?;
        schedule.add_systems(print_config);

        // bind the server
        let addr: SocketAddr = (config.host, config.port).into();
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
                    tokio::spawn(handle_connection(sock, addr));
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

fn print_config(config: Res<Config>) {
    if config.is_changed() {
        println!("{:?}", config);
    }
}

async fn handle_connection(sock: TcpStream, addr: SocketAddr) {
    debug!(addr = %addr, "new connection established");
    let (mut reader, writer) = sock.into_split();

    loop {
        let Ok(packet) = RawPacket::decode(&mut reader).await else {
            break;
        };

        println!("{:?}", packet);
    }

    debug!(addr = %addr, "connection closed");
}
