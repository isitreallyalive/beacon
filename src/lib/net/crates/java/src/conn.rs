use std::net::{SocketAddr, TcpStream};

use beacon_net::Connection;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct JavaConnection {
    pub(crate) conn: TcpStream,
    pub(crate) addr: SocketAddr,
}

impl Connection for JavaConnection {
    type Listener = super::JavaListener;

    fn process(
        _connections: Query<(Entity, &mut Self)>,
        _listener: Option<Res<Self::Listener>>,
        _commands: Commands,
    ) -> Result<()> {
        Ok(())
    }
}

impl std::ops::Deref for JavaConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
