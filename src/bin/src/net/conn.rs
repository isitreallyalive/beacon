use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    ops::Deref,
};

use bevy_ecs::{component::Mutable, prelude::*};

mod game;
pub use game::GameConnection;

mod msmp;
pub use msmp::MsmpConnection;

mod rcon;
pub use rcon::RconConnection;

pub(crate) trait Connection: Component<Mutability = Mutable> + Sized {
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
