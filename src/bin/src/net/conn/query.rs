use std::net::UdpSocket;

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct QueryConnection(UdpSocket);

impl QueryConnection {
    pub fn new(conn: UdpSocket) -> Self {
        Self(conn)
    }

    pub fn handle(mut connections: Query<(Entity, &mut Self)>, commands: Commands) {
        for (entity, mut conn) in connections.iter_mut() {}
    }
}

impl std::ops::Deref for QueryConnection {
    type Target = UdpSocket;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
