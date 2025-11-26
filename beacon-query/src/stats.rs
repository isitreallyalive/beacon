use beacon_config::Config;

use crate::string::CString;

/// Cached server statistics
pub(crate) struct StatsCache {
    pub motd: CString,
    pub map: CString,
    pub num_players: CString,
    pub max_players: CString,
    pub host_port: u16,
    pub host_ip: CString,
}

impl From<Config> for StatsCache {
    fn from(config: Config) -> Self {
        Self {
            motd: CString::new(config.server.motd).unwrap_or_default(),
            map: CString::new(config.world.name).unwrap_or_default(),
            num_players: CString::new("0").unwrap_or_default(),
            max_players: CString::new(config.server.max_players.to_string()).unwrap_or_default(),
            host_port: config.server.port,
            host_ip: CString::new(config.server.ip.to_string()).unwrap_or_default(),
        }
    }
}

impl StatsCache {
    pub fn update(&mut self, config: &Config) {
        self.motd = CString::new(config.server.motd.clone()).unwrap_or_default();
        self.map = CString::new(config.world.name.clone()).unwrap_or_default();
        // todo: update num_players from server
        self.max_players = CString::new(config.server.max_players.to_string()).unwrap_or_default();
        self.host_port = config.server.port;
        self.host_ip = CString::new(config.server.ip.to_string()).unwrap_or_default();
    }
}
