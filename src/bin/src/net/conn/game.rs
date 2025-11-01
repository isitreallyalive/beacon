use std::net::{SocketAddr, TcpStream};

use bevy_ecs::prelude::*;

use crate::net::listen::GameListener;

#[derive(Component)]
pub struct GameConnection {
    conn: TcpStream,
    addr: SocketAddr,
}

impl super::Connection for GameConnection {
    type Listener = GameListener;

    fn new(conn: TcpStream, addr: SocketAddr) -> Self {
        Self { conn, addr }
    }

    fn handle(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, mut conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for GameConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
