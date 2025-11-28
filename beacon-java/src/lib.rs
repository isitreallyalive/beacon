use beacon_config::Config;
use kameo::prelude::*;
use tokio::net::TcpListener;

use crate::conn::JavaConnActor;

#[macro_use]
extern crate tracing;

mod conn;

pub struct JavaActor {
    listener: TcpListener,
    connections: Vec<JavaConnActor>,
}

impl Actor for JavaActor {
    type Args = Config;
    type Error = std::io::Error;

    async fn on_start(config: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        let listener = TcpListener::bind((config.server.ip, config.server.port)).await?;
        info!("listening on {}/tcp", listener.local_addr()?);
        Ok(JavaActor {
            listener,
            connections: Vec::new(),
        })
    }
}

#[messages]
impl JavaActor {
    #[message]
    pub fn get_player_count(&self) -> usize {
        self.connections.len()
    }
}

impl Message<Config> for JavaActor {
    type Reply = ();

    async fn handle(&mut self, msg: Config, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // rebind socket if address changed
        let old_addr = self.listener.local_addr().ok();
        let new_addr = (msg.server.ip, msg.server.port).into();

        if old_addr != Some(new_addr) {
            let Ok(listener) = TcpListener::bind(new_addr).await else {
                return;
            };
            self.listener = listener;

            info!("address changed: {:?} -> {:?}", old_addr.unwrap(), new_addr);
        }
    }
}
