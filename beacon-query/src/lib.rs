//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use beacon_config::{Config, ConfigUpdate};
use kameo::{message::StreamMessage, prelude::*};

use crate::sock::{UdpMessage, UdpSocket};

#[macro_use]
extern crate tracing;

mod req;
mod sock;

pub struct QueryActor {
    sock: UdpSocket,
}

impl Actor for QueryActor {
    type Args = Config;
    type Error = std::io::Error;

    async fn on_start(config: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port), &actor_ref).await?;
        Ok(Self { sock })
    }
}

impl Message<UdpMessage> for QueryActor {
    type Reply = ();

    async fn handle(&mut self, msg: UdpMessage, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let StreamMessage::Next((packet, addr)) = msg else {
            return;
        };
        info!("{:?}", packet);
    }
}

impl Message<ConfigUpdate> for QueryActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ConfigUpdate,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // rebind socket if address changed
        let old_addr = self.sock.local_addr().ok();
        let new_addr = Some((msg.query.ip, msg.query.port).into());

        if old_addr != new_addr {
            let Ok(sock) = UdpSocket::bind((msg.query.ip, msg.query.port), ctx.actor_ref()).await
            else {
                return;
            };
            self.sock = sock;

            info!(
                "query address changed: {:?} -> {:?}",
                old_addr.unwrap(),
                new_addr.unwrap()
            );
        }
    }
}
