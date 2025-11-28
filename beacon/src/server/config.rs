use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use beacon::BeaconError;
use beacon_config::{Config, ConfigError, ConfigUpdate};
use kameo::prelude::*;
use notify::{
    EventKind, Watcher,
    event::{ModifyKind, RenameMode},
};

use crate::server::BeaconActor;

const DEDUP: Duration = Duration::from_millis(500);
const DEBOUNCE: Duration = Duration::from_millis(100);

/// Manages configuration.
pub(crate) struct BeaconConfig {
    data: Config,
    path: Arc<RwLock<PathBuf>>,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
    last_modify: Instant,
}

impl BeaconConfig {
    pub fn new(path: PathBuf, actor: ActorRef<BeaconActor>) -> Result<Self, BeaconError> {
        // load initial data
        let data = Config::read(&path)?;

        // start watcher
        let path = Arc::new(RwLock::new(path.canonicalize()?));
        let mut watcher = {
            let path = path.clone();
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                // only send events for the config file
                if let Ok(event) = res {
                    if let Ok(path) = path.read()
                        && event.paths.contains(&path)
                    {
                        let _ = actor.tell(WatcherEvent(event)).try_send();
                    }
                }
            })?
        };

        // resolve parent directory
        let parent = path
            .read()?
            .parent()
            .ok_or(BeaconError::Other("invalid config path".into()))?
            .to_path_buf();

        watcher.watch(&parent, notify::RecursiveMode::Recursive)?;

        Ok(Self {
            data,
            path,
            watcher,
            last_modify: Instant::now(),
        })
    }
}

impl std::ops::Deref for BeaconConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct WatcherEvent(notify::Event);

impl Message<WatcherEvent> for BeaconActor {
    type Reply = Result<(), BeaconError>;

    async fn handle(
        &mut self,
        WatcherEvent(event): WatcherEvent,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match event.kind {
            // config was created/modified, reload
            EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(_) => {
                // dedup modify events
                if matches!(event.kind, EventKind::Modify(_)) {
                    let now = Instant::now();
                    if now.duration_since(self.config.last_modify) < DEDUP {
                        return Ok(());
                    }
                    self.config.last_modify = now;
                }

                let new = {
                    let path = self.config.path.read()?;
                    match Config::read(&path) {
                        Ok(config) => config,
                        Err(err) => {
                            error!("failed to parse config, keeping old.\n{err}");
                            return Ok(());
                        }
                    }
                };

                if new != self.config.data {
                    // sync actors
                    self.config.data = new;
                    self.sync();

                    // send out config updates
                    // send the event after debouncing
                    tokio::time::sleep(DEBOUNCE).await;
                    if let Some(query) = &self.query {
                        query
                            .tell(ConfigUpdate(self.config.clone()))
                            .await
                            .map_err(ConfigError::from)?;
                    }
                }
            }
            // config file was renamed, update path
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                if let Some(new) = event.paths.last() {
                    let mut path = self.config.path.write()?;
                    *path = new.clone();
                }
            }
            _ => {}
        }

        Ok(())
    }
}
