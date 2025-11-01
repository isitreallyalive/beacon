use std::{
    io,
    net::{TcpListener, UdpSocket},
    ops::Deref,
};

use beacon_config::HasListener;
use bevy_ecs::prelude::*;

pub trait Listener {
    fn setup<C: HasListener>(world: &mut World, config: &C) -> io::Result<()>;
}

macro_rules! define {
    ($name:ident, $target:ty) => {
        #[derive(Resource)]
        pub struct $name($target);

        impl Listener for $name {
            fn setup<C: HasListener>(world: &mut World, config: &C) -> io::Result<()> {
                let listener = <$target>::bind(config.address())?;
                world.insert_resource($name(listener));
                Ok(())
            }
        }

        impl Deref for $name {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

define!(Game, TcpListener);
define!(Rcon, TcpListener);
define!(Query, UdpSocket);
// todo: msmp websocket (tungstenite + tokio?)
