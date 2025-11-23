use std::io;

pub use async_trait::async_trait;

#[async_trait]
pub trait Tickable: Sized + Send + Sync + 'static {
    async fn tick(&mut self) -> io::Result<()>;
}
