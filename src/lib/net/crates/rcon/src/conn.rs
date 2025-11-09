use std::net::{SocketAddr, TcpStream};

use beacon_net::Connection;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct RconConnection {
    pub(crate) conn: TcpStream,
    pub(crate) addr: SocketAddr,
}

impl Connection for RconConnection {
    type Listener = super::RconListener;

    fn process(
        _connections: Query<(Entity, &mut Self)>,
        _listener: Option<Res<Self::Listener>>,
        _commands: Commands,
    ) -> Result<()> {
        Ok(())
    }
}

impl std::ops::Deref for RconConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
