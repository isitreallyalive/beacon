use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use kameo::prelude::*;
use kameo_actors::pubsub::{PubSub, Publish};
use notify::Watcher;

use crate::read::ConfigData;

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
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ConfigActor {
    path: PathBuf,
    update_pub: ActorRef<PubSub<ConfigUpdate>>,
    data: ConfigData,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

/// Internal message to trigger config reload
struct ReloadConfig;

/// Message published when config is updated
#[derive(Clone)]
pub struct ConfigUpdate;

impl Actor for ConfigActor {
    type Args = (PathBuf, ActorRef<PubSub<ConfigUpdate>>);
    type Error = ConfigError;

    async fn on_start(
        (path, update_pub): Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        // read initial config
        let data = ConfigData::read(&path)?;

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
                        if let Some(last) = LAST {
                            if now.duration_since(last) < Duration::from_millis(100) {
                                return;
                            }
                        }
                        LAST = Some(now);
                    }

                    // send the event
                    let _ = actor_ref.tell(ReloadConfig).try_send();
                }
            })?;

        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            path,
            update_pub,
            data,
            watcher,
        })
    }
}

impl Message<ReloadConfig> for ConfigActor {
    type Reply = ();

    async fn handle(&mut self, _: ReloadConfig, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        debug!("reloading config from {:?}", self.path);

        // reload config and inform subscribers
        let _ = self.data.reload(&self.path);
        let _ = self.update_pub.tell(Publish(ConfigUpdate)).await;
    }
}
