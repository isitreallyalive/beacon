//! Query protocol implementation.
//!
//! See: https://minecraft.wiki/w/Query

use std::{collections::HashMap, net::SocketAddr, time::Duration};

use beacon_config::{Config, ConfigUpdate};
use deku::DekuContainerWrite;
use kameo::{message::StreamMessage, prelude::*};
use kameo_actors::scheduler::{Scheduler, SetInterval};

use crate::{
    kv::KeyValue,
    req::{QueryRequest, StatRequest},
    res::*,
    sock::{UdpMessage, UdpSocket},
    stats::StatsCache,
    string::CString,
};

#[macro_use]
extern crate tracing;

mod kv;
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

/// Message to clear stored challenge tokens.
#[derive(Clone)]
struct ClearTokens;

pub struct QueryActor {
    sock: UdpSocket,
    stats: StatsCache,
    tokens: HashMap<SocketAddr, i32>,
}

impl QueryActor {
    async fn handle(
        &mut self,
        req: QueryRequest,
        addr: SocketAddr,
    ) -> Result<QueryResponse, QueryError> {
        Ok(match req {
            QueryRequest::Handshake { session_id } => {
                // generate and store a challenge token
                let challenge_token = {
                    let value: i32 = rand::random();
                    self.tokens.insert(addr, value);
                    CString::new(value.to_string())?
                };

                QueryResponse::Handshake {
                    session_id,
                    challenge_token,
                }
            }
            QueryRequest::Stat(StatRequest {
                session_id,
                challenge_token,
                full,
            }) => {
                // validate challenge token
                let expected_token = self.tokens.get(&addr);
                if expected_token != Some(&challenge_token) {
                    warn!(
                        expected = expected_token,
                        got = challenge_token,
                        "invalid challenge token from {}",
                        addr
                    );
                    return Err(QueryError::InvalidToken(addr));
                }

                if full {
                    QueryResponse::FullStat {
                        session_id,
                        kv: {
                            let mut kv = KeyValue::default();
                            kv.insert(HOSTNAME_KEY.clone(), self.stats.motd.clone());
                            kv.insert(GAMETYPE_KEY.clone(), GAME_TYPE.clone());
                            kv.insert(GAME_ID_KEY.clone(), GAME_ID.clone());
                            kv.insert(VERSION_KEY.clone(), VERSION.clone());
                            kv.insert(PLUGINS_KEY.clone(), PLUGINS.clone());
                            kv.insert(MAP_KEY.clone(), self.stats.map.clone());
                            kv.insert(NUMPLAYERS_KEY.clone(), self.stats.num_players.clone());
                            kv.insert(MAXPLAYERS_KEY.clone(), self.stats.max_players.clone());
                            kv.insert(
                                HOSTPORT_KEY.clone(),
                                CString::new(self.stats.host_port.to_string())?,
                            );
                            kv.insert(HOSTIP_KEY.clone(), self.stats.host_ip.clone());
                            kv
                        },
                        // todo: fetch player list from server
                        players: vec![],
                    }
                } else {
                    QueryResponse::BasicStat {
                        session_id,
                        motd: self.stats.motd.clone(),
                        game_type: GAME_TYPE.clone(),
                        map: self.stats.map.clone(),
                        num_players: self.stats.num_players.clone(),
                        max_players: self.stats.max_players.clone(),
                        host_port: self.stats.host_port,
                        host_ip: self.stats.host_ip.clone(),
                    }
                }
            }
        })
    }
}

impl Actor for QueryActor {
    type Args = (Config, ActorRef<Scheduler>);
    type Error = QueryError;

    async fn on_start(
        (config, scheduler): Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port), &actor_ref).await?;
        let stats = StatsCache::from(config);

        // clear tokens periodically
        let _ = scheduler
            .tell(SetInterval::new(
                actor_ref.downgrade(),
                CLEAR_INTERVAL,
                ClearTokens,
            ))
            .await;

        Ok(Self {
            sock,
            stats,
            tokens: HashMap::new(),
        })
    }
}

impl Message<UdpMessage> for QueryActor {
    type Reply = ();

    async fn handle(&mut self, msg: UdpMessage, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let StreamMessage::Next((req, addr)) = msg else {
            return;
        };

        let Ok(Ok(res)) = self.handle(req, addr).await.map(|res| res.to_bytes()) else {
            return;
        };
        let _ = self.sock.send_to(&res, addr).await;
    }
}

impl Message<ClearTokens> for QueryActor {
    type Reply = ();

    async fn handle(&mut self, _: ClearTokens, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let size = self.tokens.len();
        if size > 0 {
            self.tokens.clear();
            debug!("cleared {size} challenge tokens");
        }
    }
}

impl Message<ConfigUpdate> for QueryActor {
    type Reply = ();

    async fn handle(
        &mut self,
        ConfigUpdate(config): ConfigUpdate,
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

            info!(
                "query address changed: {:?} -> {:?}",
                old_addr.unwrap(),
                new_addr
            );
        }
    }
}
