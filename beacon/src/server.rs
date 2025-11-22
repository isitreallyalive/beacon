use std::io;

pub struct Beacon {
    // game: TcpListener,
    query: beacon_query::QueryHandler,
    // rcon: TcpListener,
    // msmp: TcpListener
}

impl Beacon {
    pub async fn new() -> io::Result<Self> {
        let query = beacon_query::QueryHandler::new().await?;
        Ok(Self { query })
    }

    pub async fn start(self) {
        loop {
            tokio::select! {
                res = self.query.tick() => {
                    if let Err(err) = res {
                        error!("{:?}", err);
                    }
                }
            }
        }
    }
}
