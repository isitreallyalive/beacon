//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::{collections::HashMap, net::SocketAddr, time::Duration};

use beacon_config::Config;
use beacon_java::JavaActor;
use deku::DekuContainerWrite;
use kameo::{message::StreamMessage, prelude::*};
use kameo_actors::scheduler::{Scheduler, SetTimeout};

use crate::{
    sock::{UdpMessage, UdpSocket},
    stats::StatsCache,
};

#[macro_use]
extern crate tracing;

mod kv;
mod process;
mod req;
mod res;
mod sock;
mod stats;
mod string;

/// How often to clear challenge tokens.
const CLEAR_INTERVAL: Duration = Duration::from_secs(30);

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
    stats: StatsCache,
    tokens: HashMap<SocketAddr, i32>,
    java: ActorRef<JavaActor>,
}

impl Actor for QueryActor {
    type Args = (Config, ActorRef<JavaActor>, ActorRef<Scheduler>);
    type Error = QueryError;

    async fn on_start(
        (config, java, scheduler): Self::Args,
        actor: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port), &actor).await?;
        let stats = StatsCache::from(config);

        // clear tokens periodically
        let _ = scheduler
            .tell(SetTimeout::new(
                actor.downgrade(),
                CLEAR_INTERVAL,
                ClearTokens,
            ))
            .await;

        info!("listening on {}/udp", sock.local_addr()?);

        Ok(Self {
            sock,
            stats,
            tokens: HashMap::new(),
            java,
        })
    }

    async fn on_stop(
        &mut self,
        _: WeakActorRef<Self>,
        reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
        if let ActorStopReason::Killed = reason {
            info!("query stopped")
        }
        Ok(())
    }
}

#[messages]
impl QueryActor {
    /// Clear all stored challenge tokens.
    #[message]
    fn clear_tokens(&mut self) {
        let size = self.tokens.len();
        if size > 0 {
            self.tokens.clear();
            debug!("cleared {size} challenge tokens");
        }
    }
}

impl Message<UdpMessage> for QueryActor {
    type Reply = ();

    async fn handle(&mut self, msg: UdpMessage, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let StreamMessage::Next((req, addr)) = msg else {
            return;
        };

        let Ok(Ok(res)) = self.process(req, addr).await.map(|res| res.to_bytes()) else {
            return;
        };
        let _ = self.sock.send_to(&res, addr).await;
    }
}

impl Message<Config> for QueryActor {
    type Reply = ();

    async fn handle(
        &mut self,
        config: Config,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.stats.update(&config);

        // rebind socket if address changed
        let old_addr = self.sock.local_addr().ok();
        let new_addr = (config.query.ip, config.query.port).into();

        if old_addr != Some(new_addr) {
            let Ok(sock) = UdpSocket::bind(new_addr, ctx.actor_ref()).await else {
                return;
            };
            self.sock = sock;

            info!("address changed: {:?} -> {:?}", old_addr.unwrap(), new_addr);
        }
    }
}
