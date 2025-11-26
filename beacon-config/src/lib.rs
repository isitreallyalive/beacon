use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use kameo::prelude::*;
use kameo_actors::pubsub::{PubSub, Publish};
use notify::Watcher;

pub use crate::read::Config;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate tracing;

mod def;
mod read;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("file watcher error: {0}")]
    Watcher(#[from] notify::Error),
    #[error("failed to read config: {0}")]
    Read(#[from] toml::de::Error),
    #[error("failed to write config: {0}")]
    Write(#[from] toml::ser::Error),
    #[error("failed to subscribe to config updates")]
    Subscribe(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Message to trigger a config reload
struct ReloadConfig;

/// Message to notify subscribers of config updates
#[derive(Clone)]
pub struct ConfigUpdate(pub Config);

/// Manages configuration file hot reloading.
pub struct ConfigActor {
    data: Config,
    path: PathBuf,
    config_update: ActorRef<PubSub<ConfigUpdate>>,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

impl Actor for ConfigActor {
    type Args = (Config, PathBuf, ActorRef<PubSub<ConfigUpdate>>);
    type Error = ConfigError;

    async fn on_start(
        (data, path, config_update): Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        // create file watcher
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                static mut LAST: Option<Instant> = None;

                let Ok(event) = res else {
                    return;
                };
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    // deduplicate events
                    let now = Instant::now();
                    unsafe {
                        if let Some(last) = LAST
                            && now.duration_since(last) < Duration::from_millis(500)
                        {
                            return;
                        }
                        LAST = Some(now);
                    }

                    // send the event after debouncing
                    std::thread::sleep(Duration::from_millis(100));
                    let _ = actor_ref.tell(ReloadConfig).try_send();
                }
            })?;

        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            data,
            path,
            config_update,
            watcher,
        })
    }
}

impl Message<ReloadConfig> for ConfigActor {
    type Reply = ();

    async fn handle(&mut self, _: ReloadConfig, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // reload config and inform subscribers
        let _ = self.data.reload(&self.path);
        let _ = self
            .config_update
            .tell(Publish(ConfigUpdate(self.data.clone())))
            .await;
    }
}
