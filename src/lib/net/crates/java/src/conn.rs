use std::net::{SocketAddr, TcpStream};

use beacon_net::Connection;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct JavaConnection {
    pub(crate) conn: TcpStream,
    pub(crate) addr: SocketAddr,
}

impl Connection for JavaConnection {
    fn process(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for JavaConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
