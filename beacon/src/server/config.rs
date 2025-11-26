use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use beacon::BeaconError;
use beacon_config::{Config, ConfigUpdate};
use kameo::prelude::*;
use notify::Watcher;

use crate::server::BeaconActor;

const DEDUP: Duration = Duration::from_millis(500);
const DEBOUNCE: Duration = Duration::from_millis(100);

/// Manages configuration.
pub(crate) struct BeaconConfig {
    data: Config,
    path: PathBuf,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

impl BeaconConfig {
    pub fn new(path: PathBuf, actor: ActorRef<BeaconActor>) -> Result<Self, BeaconError> {
        // load initial data
        let data = Config::read(&path);

        // start watcher
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
                            && now.duration_since(last) < DEDUP
                        {
                            return;
                        }
                        LAST = Some(now);
                    }

                    // send the event after debouncing
                    std::thread::sleep(DEBOUNCE);
                    let _ = actor.tell(ReloadConfig).try_send();
                }
            })?;

        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            data,
            path,
            watcher,
        })
    }
}

impl std::ops::Deref for BeaconConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Message to trigger a config reload
struct ReloadConfig;

impl Message<ReloadConfig> for BeaconActor {
    type Reply = Result<(), BeaconError>;

    async fn handle(&mut self, _: ReloadConfig, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let new_config = Config::read(&self.config.path);

        if new_config != self.config.data {
            // sync actors
            self.config.data = new_config;
            self.sync();

            // send out config updates
            if let Some(query) = &self.query {
                query.tell(ConfigUpdate(self.config.clone())).await?;
            }
        }

        Ok(())
    }
}
