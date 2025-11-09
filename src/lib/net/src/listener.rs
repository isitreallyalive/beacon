use std::{io, ops::Deref};

use beacon_config::Config;
use bevy_ecs::prelude::*;

pub trait Listener: Deref + Resource + Sized {
    const NAME: &str;

    /// Register the listener as a resource and relevant systems to schedule.
    fn register(world: &mut World, schedule: &mut Schedule, config: &Config) -> io::Result<()> {
        world.insert_resource(Self::new(config)?);
        schedule.add_systems((Self::update, Self::accept));
        Ok(())
    }

    /// Create a new listener based on the provided configuration.
    fn new(config: &Config) -> io::Result<Self>;

    /// Update the listener based on configuration changes.
    fn update(
        config: Res<Config>,
        listener: Option<ResMut<Self>>,
        commands: Commands,
    ) -> Result<()>;

    /// Accept new connections.
    fn accept(listener: Option<Res<Self>>, commands: Commands);
}

#[macro_export]
macro_rules! update_listener {
    ($field:ident) => {
        fn update(config: Res<Config>, mut listener: Option<ResMut<Self>>, mut commands: Commands) -> Result<()> {
            update_listener!(@ listener, config, $field, commands);
            Ok(())
        }
    };
    (@toggle $field:ident) => {
        fn update(config: Res<Config>, mut listener: Option<ResMut<Self>>, mut commands: Commands) -> Result<()> {
            if config.$field.enable {
                update_listener!(@ listener, config, $field, commands);
            } else {
                commands.remove_resource::<Self>();
            }
            Ok(())
        }
    };
    (@ $listener:expr, $config:expr, $field:ident, $commands:expr) => {
        let new_addr = ($config.$field.ip, $config.$field.port).into();
        let replace = match $listener {
            Some(existing) => existing.local_addr().ok() != Some(new_addr),
            None => true,
        };
        if replace {
            let resource = Self::new(&$config)?;
            $commands.insert_resource(resource);
        }
    };
}

#[macro_export]
macro_rules! accept_tcp {
    ($conn:ident) => {
        fn accept(listener: Option<Res<Self>>, mut commands: Commands) {
            if let Some(listener) = listener {
                match listener.accept() {
                    Ok((conn, addr)) => {
                        info!(service = Self::NAME, addr = ?addr, "accepted connection");
                        commands.spawn($conn {
                            conn,
                            addr
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // no incoming connection, non-blocking
                    }
                    Err(e) => {
                        error!(service = Self::NAME, error = %e, "failed to accept connection");
                    }
                }
            }
        }
    };
}
