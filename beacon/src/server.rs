use std::io;

use beacon_config::Config;

pub struct Beacon {
    config: Config,
    // server: TcpListener,
    query: beacon_query::QueryHandler,
    // rcon: TcpListener,
    // msmp: TcpListener
}

impl Beacon {
    pub async fn new() -> io::Result<Self> {
        let config = Config::read("beacon.toml".into()).await;
        let query = beacon_query::QueryHandler::new(config.data.clone()).await?;
        Ok(Self { config, query })
    }

    pub async fn start(mut self) {
        loop {
            tokio::select! {
                _ = self.config.tick() => {}
                res = self.query.tick() => {
                    if let Err(err) = res {
                        println!("{:?}", err);
                    }
                }
            }
        }
    }
}
