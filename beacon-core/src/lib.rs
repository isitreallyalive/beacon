//! # beacon-core

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use miette::{IntoDiagnostic, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

/// The Minecraft server you'll love.
pub struct BeaconServer {
    listener: TcpListener,
    state: Arc<ServerState>,
}

/// Shared server state.
#[derive(Default)]
pub struct ServerState {
    cancel_token: CancellationToken,
}

impl BeaconServer {
    /// Create a new instance of beacon.
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 25565))
            .await
            .into_diagnostic()?;
        Ok(Self {
            listener,
            state: Arc::new(ServerState::default()),
        })
    }

    /// Start the server.
    pub async fn start(&self) -> Result<()> {
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
                Ok((sock, addr)) = self.listener.accept() => handle_connection(sock, addr).await,
                _ = self.state.cancel_token.cancelled() => break,
            }
        }

        // shutdown
        println!("shutting down...");

        Ok(())
    }
}

async fn handle_connection(sock: TcpStream, addr: SocketAddr) {}
