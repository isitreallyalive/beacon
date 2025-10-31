use std::net::TcpListener;

use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Listener(TcpListener);

impl std::ops::Deref for Listener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Listener {
    /// Bind a TCP listener to the specified port on all interfaces.
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        Ok(Listener(listener))
    }
}
