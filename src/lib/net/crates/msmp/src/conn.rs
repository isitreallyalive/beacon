use std::net::{SocketAddr, TcpStream};

use beacon_net::Connection;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct MsmpConnection {
    conn: TcpStream,
    addr: SocketAddr,
}

impl Connection for MsmpConnection {
    fn process(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for MsmpConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
