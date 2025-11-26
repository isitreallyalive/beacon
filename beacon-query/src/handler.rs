use std::{collections::HashMap, net::SocketAddr};

use beacon_config::Config;

use crate::{
    QueryError,
    kv::KeyValue,
    req::{QueryRequest, StatRequest},
    res::*,
    string::CString,
};

pub struct QueryHandler {
    tokens: HashMap<SocketAddr, i32>,

    // cached statistics
    motd: CString,
    map: CString,
    num_players: CString,
    max_players: CString,
    host_port: u16,
    host_ip: CString,
}

impl From<Config> for QueryHandler {
    fn from(config: Config) -> Self {
        Self {
            tokens: HashMap::new(),
            motd: CString::new(config.server.motd).unwrap_or_default(),
            map: CString::new(config.world.name).unwrap_or_default(),
            num_players: CString::new("0").unwrap_or_default(),
            max_players: CString::new(config.server.max_players.to_string()).unwrap_or_default(),
            host_port: config.server.port,
            host_ip: CString::new(config.server.ip.to_string()).unwrap_or_default(),
        }
    }
}

impl QueryHandler {
    pub async fn handle(
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
                        kv_marker: KV_MARKER,
                        kv: {
                            let mut kv = KeyValue::default();
                            kv.insert(HOSTNAME_KEY.clone(), self.motd.clone());
                            kv.insert(GAMETYPE_KEY.clone(), GAME_TYPE.clone());
                            kv.insert(GAME_ID_KEY.clone(), GAME_ID.clone());
                            kv.insert(VERSION_KEY.clone(), VERSION.clone());
                            kv.insert(PLUGINS_KEY.clone(), PLUGINS.clone());
                            kv.insert(MAP_KEY.clone(), self.map.clone());
                            kv.insert(NUMPLAYERS_KEY.clone(), self.num_players.clone());
                            kv.insert(MAXPLAYERS_KEY.clone(), self.max_players.clone());
                            kv.insert(
                                HOSTPORT_KEY.clone(),
                                CString::new(self.host_port.to_string())?,
                            );
                            kv.insert(HOSTIP_KEY.clone(), self.host_ip.clone());
                            kv
                        },
                        player_marker: PLAYER_MARKER,
                        // todo: fetch player list from server
                        players: vec![],
                        nul: 0,
                    }
                } else {
                    QueryResponse::BasicStat {
                        session_id,
                        motd: self.motd.clone(),
                        game_type: GAME_TYPE.clone(),
                        map: self.map.clone(),
                        num_players: self.num_players.clone(),
                        max_players: self.max_players.clone(),
                        host_port: self.host_port,
                        host_ip: self.host_ip.clone(),
                    }
                }
            }
        })
    }

    pub fn update_stats(&mut self, config: &Config) {
        self.motd = CString::new(config.server.motd.clone()).unwrap_or_default();
        self.map = CString::new(config.world.name.clone()).unwrap_or_default();
        // todo: update num_players from server
        self.max_players = CString::new(config.server.max_players.to_string()).unwrap_or_default();
        self.host_port = config.server.port;
        self.host_ip = CString::new(config.server.ip.to_string()).unwrap_or_default();
    }
}
