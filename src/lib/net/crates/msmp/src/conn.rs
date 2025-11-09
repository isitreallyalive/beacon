use std::{
    io,
    net::{SocketAddr, TcpStream},
};

use beacon_net::Connection;
use bevy_ecs::prelude::*;
use tungstenite::{Message, WebSocket};

#[derive(Component)]
pub struct MsmpConnection {
    pub(crate) ws: WebSocket<TcpStream>,
    pub(crate) addr: SocketAddr,
}

impl Connection for MsmpConnection {
    type Listener = super::MsmpListener;

    fn process(
        mut connections: Query<(Entity, &mut Self)>,
        listener: Option<Res<Self::Listener>>,
        mut commands: Commands,
    ) -> Result<()> {
        let Some(listener) = listener else {
            return Ok(());
        };

        for (entity, mut conn) in connections.iter_mut() {
            let message = match conn.ws.read() {
                Ok(msg) => msg,
                Err(tungstenite::Error::Io(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(tungstenite::Error::ConnectionClosed) => {
                    debug!(addr = %conn.addr, "connection closed, despawning");
                    commands.entity(entity).despawn();
                    continue;
                }
                Err(e) => {
                    error!(addr = %conn.addr, error = %e, "connection error, closing");
                    commands.entity(entity).despawn();
                    continue;
                }
            };

            if let Message::Text(bytes) = message {
                if let Some(response) = listener.io.handle_request_sync(&bytes) {
                    conn.ws.write(Message::Text(response.into()))?;
                    conn.ws.flush()?;
                }
            }
        }

        Ok(())
    }
}
