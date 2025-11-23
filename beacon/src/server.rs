use std::{io, time::Duration};

use beacon_config::Config;
use beacon_query::QueryHandler;
use beacon_util::Tickable;

const TICK_DURATION: Duration = Duration::from_millis(50);

pub struct Beacon {
    config: Config,
    // server: TcpListener,
    query: QueryHandler,
    // rcon: TcpListener,
    // msmp: TcpListener
}

impl Beacon {
    pub async fn new() -> io::Result<Self> {
        let config = Config::read("beacon.toml".into()).await;
        let query = QueryHandler::new(&config).await?;
        Ok(Self { config, query })
    }

    pub async fn start(mut self) -> io::Result<()> {
        loop {
            self.config.tick().await?;
            self.query.tick().await?;
            tokio::time::sleep(TICK_DURATION).await;
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}
