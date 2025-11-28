use kameo::prelude::*;
use tokio::net::TcpStream;

pub struct JavaConnActor {
    stream: TcpStream,
}

impl Actor for JavaConnActor {
    type Args = TcpStream;
    type Error = ();

    async fn on_start(stream: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(JavaConnActor { stream })
    }
}
