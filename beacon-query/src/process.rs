use std::net::SocketAddr;

use beacon_java::GetOnline;

use crate::{
    QueryActor, QueryError,
    kv::KeyValue,
    req::{QueryRequest, StatRequest},
    res::*,
    string::CString,
};

impl QueryActor {
    /// Process an incoming query request.
    pub(crate) async fn process(
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

                // get the current number of players from the Java server
                let num_players =
                    CString::new(self.java.ask(GetOnline).await.unwrap_or(0).to_string())?;

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
                            kv.insert(NUMPLAYERS_KEY.clone(), num_players);
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
                        num_players,
                        max_players: self.stats.max_players.clone(),
                        host_port: self.stats.host_port,
                        host_ip: self.stats.host_ip.clone(),
                    }
                }
            }
        })
    }
}
