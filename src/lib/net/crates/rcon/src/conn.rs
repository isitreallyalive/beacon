use std::net::{SocketAddr, TcpStream};

use beacon_net::Connection;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct RconConnection {
    conn: TcpStream,
    addr: SocketAddr,
}

impl Connection for RconConnection {
    const SERVICE_NAME: &'static str = "RCON";
    type Listener = crate::RconListener;

    fn new(conn: TcpStream, addr: SocketAddr) -> Self {
        Self { conn, addr }
    }

    fn handle(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for RconConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
