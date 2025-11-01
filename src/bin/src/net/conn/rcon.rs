use std::net::{SocketAddr, TcpStream};

use bevy_ecs::prelude::*;

use crate::net::listen::RconListener;

#[derive(Component)]
pub struct RconConnection {
    conn: TcpStream,
    addr: SocketAddr,
}

impl super::TcpConnection for RconConnection {
    type Listener = RconListener;

    fn new(conn: TcpStream, addr: SocketAddr) -> Self {
        Self { conn, addr }
    }

    fn handle(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, mut conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for RconConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
