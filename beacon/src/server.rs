use std::{io, rc::Rc};

use beacon_config::Config;
use tokio::sync::Mutex;

pub struct Beacon {
    // server: TcpListener,
    query: beacon_query::QueryHandler,
    // rcon: TcpListener,
    // msmp: TcpListener
}

impl Beacon {
    pub async fn new() -> io::Result<Self> {
        let config = Rc::new(Mutex::new(Config::default()));
        let query = beacon_query::QueryHandler::new(config).await?;
        Ok(Self { query })
    }

    pub async fn start(mut self) {
        loop {
            tokio::select! {
                res = self.query.tick() => {
                    if let Err(err) = res {
                        println!("{:?}", err);
                    }
                }
            }
        }
    }
}
