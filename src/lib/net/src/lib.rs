use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    ops::Deref,
};

use beacon_config::Config;
use bevy_ecs::{component::Mutable, prelude::*};

pub trait Listener: Deref + Resource + Sized {
    fn register(world: &mut World, schedule: &mut Schedule, config: &Config) -> io::Result<()> {
        world.insert_resource(Self::new(config)?);
        schedule.add_systems(Self::update);
        Ok(())
    }
    fn new(config: &Config) -> io::Result<Self>;

    /// Update the listener based on configuration changes.
    fn update(
        config: Res<Config>,
        listener: Option<ResMut<Self>>,
        commands: Commands,
    ) -> Result<()>;
}

#[macro_export]
macro_rules! update_listener {
    ($listener:ty: $field:ident) => {
        fn update(config: Res<Config>, mut listener: Option<ResMut<$listener>>, mut commands: Commands) -> Result<()> {
            update_listener!(@ listener, $listener, config, $field, commands);
            Ok(())
        }
    };
    (@toggle $listener:ty: $field:ident) => {
        fn update(config: Res<Config>, mut listener: Option<ResMut<$listener>>, mut commands: Commands) -> Result<()> {
            if config.$field.enable {
                update_listener!(@ listener, $listener, $config, $field, commands);
            } else {
                commands.remove_resource::<$listener>();
            }
            Ok(())
        }
    };
    (@ $listener_val:expr, $listener_ty: ty, $config:expr, $field:ident, $commands:expr) => {
        let new_addr = ($config.$field.ip, $config.$field.port).into();
        let replace = match $listener_val {
            Some(existing) => existing.local_addr().ok() != Some(new_addr),
            None => true,
        };
        if replace {
            let resource = <$listener_ty>::new(&$config)?;
            $commands.insert_resource(resource);
        }
    };
}

pub trait Connection: Component<Mutability = Mutable> + Sized {
    // todo: update
    type Listener: Deref<Target = TcpListener> + Resource;

    fn new(conn: TcpStream, addr: SocketAddr) -> Self;

    fn register(schedule: &mut Schedule) {
        schedule.add_systems((Self::accept, Self::handle));
    }

    /// System to accept new connections and spawn them as entities.
    fn accept(listener: Option<Res<Self::Listener>>, mut commands: Commands) {
        if let Some(listener) = listener {
            match listener.accept() {
                Ok((conn, addr)) => {
                    commands.spawn(Self::new(conn, addr));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // no incoming connection, non-blocking
                }
                Err(e) => {
                    eprintln!("failed to accept connection: {}", e);
                }
            }
        }
    }

    /// System to process existing connections.
    fn handle(connections: Query<(Entity, &mut Self)>, commands: Commands);
}
