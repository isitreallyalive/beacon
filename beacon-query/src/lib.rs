//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::net::SocketAddr;

use beacon_config::{Config, ConfigUpdate};
use deku::DekuContainerWrite;
use kameo::{message::StreamMessage, prelude::*};

use crate::{
    handler::QueryHandler,
    sock::{UdpMessage, UdpSocket},
};

#[macro_use]
extern crate tracing;

mod handler;
mod kv;
mod req;
mod res;
mod sock;
mod string;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("{0} provided an invalid challenge token")]
    InvalidToken(SocketAddr),
    #[error("Nul error: {0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct QueryActor {
    sock: UdpSocket,
    handler: QueryHandler,
}

impl Actor for QueryActor {
    type Args = Config;
    type Error = std::io::Error;

    async fn on_start(config: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port), &actor_ref).await?;
        let handler = QueryHandler::from(config);
        Ok(Self { sock, handler })
    }
}

impl Message<UdpMessage> for QueryActor {
    type Reply = ();

    async fn handle(&mut self, msg: UdpMessage, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let StreamMessage::Next((req, addr)) = msg else {
            return;
        };
        let Ok(Ok(res)) = self
            .handler
            .handle(req, addr)
            .await
            .map(|res| res.to_bytes())
        else {
            return;
        };
        let _ = self.sock.send_to(&res, addr).await;
    }
}

impl Message<ConfigUpdate> for QueryActor {
    type Reply = ();

    async fn handle(
        &mut self,
        ConfigUpdate(config): ConfigUpdate,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handler.update_stats(&config);

        // rebind socket if address changed
        let old_addr = self.sock.local_addr().ok();
        let new_addr = (config.query.ip, config.query.port).into();

        if old_addr != Some(new_addr) {
            let Ok(sock) = UdpSocket::bind(new_addr, ctx.actor_ref()).await else {
                return;
            };
            self.sock = sock;

            info!(
                "query address changed: {:?} -> {:?}",
                old_addr.unwrap(),
                new_addr
            );
        }
    }
}
