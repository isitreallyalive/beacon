use std::{
    io,
    net::{TcpListener, UdpSocket},
};

use beacon_config::Config;
use bevy_ecs::prelude::*;

// todo: msmp websocket (tungstenite + tokio?)

macro_rules! define {
    ($struct:ident ($field:ident): $socket:ty) => {
        #[derive(Resource)]
        pub struct $struct($socket);

        impl std::ops::Deref for $struct {
            type Target = $socket;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl $struct {
            pub fn new(config: &Config) -> io::Result<Self> {
                let listener = <$socket>::bind((config.$field.ip, config.$field.port))?;
                Ok($struct(listener))
            }
        }
    };
}

define!(Game(server): TcpListener);
define!(Rcon(rcon): TcpListener);
define!(Query(query): UdpSocket);

macro_rules! update {
    ($struct:ident ($field:ident); $config:expr, $commands:expr, $world:expr) => {
        let new_addr = ($config.$field.ip, $config.$field.port).into();
        let replace = match $world.get_resource::<$struct>() {
            Some(existing) => existing.0.local_addr().ok() != Some(new_addr),
            None => true,
        };
        if replace {
            let resource = $struct::new(&$config)?;
            $commands.insert_resource(resource);
        }
    };
    (@toggle $struct:ident ($field:ident); $config:expr, $commands:expr, $world:expr) => {
        if $config.$field.enable {
            update!($struct ($field); $config, $commands, $world);
        } else {
            $commands.remove_resource::<$struct>();
        }
    };
}

/// Update listeners that have changed in config
pub fn update(config: Res<Config>, mut commands: Commands, world: &World) -> Result<()> {
    update!(Game(server); config, commands, world);
    update!(@toggle Rcon(rcon); config, commands, world);
    update!(@toggle Query(query); config, commands, world);
    Ok(())
}
