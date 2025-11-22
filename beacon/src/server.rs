use std::io;

use tokio::net::{TcpListener, UdpSocket};

pub struct Beacon {
    game: TcpListener,
    query: UdpSocket,
    // rcon: TcpListener,
    // msmp: TcpListener
}

impl Beacon {
    pub async fn new() -> io::Result<Self> {
        let game = TcpListener::bind("0.0.0.0:25565").await?;
        let query = UdpSocket::bind("0.0.0.0:25565").await?;
        Ok(Self { game, query })
    }

    pub async fn start(self) {
        let mut query_buf = [0u8; 2048];

        loop {
            tokio::select! {
                Ok((stream, addr)) = self.game.accept() => {
                }

                Ok((len, addr)) = self.query.recv_from(&mut query_buf) => {
                }
            }
        }
    }
}
