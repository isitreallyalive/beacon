use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy_ecs::prelude::*;
use flume::Receiver;
use miette::Report;
use notify::{Event, EventKind, Watcher};

use crate::{Config, ConfigError};

/// How long to wait after recieving a file change event before processing another.
const DEBOUNCE_TIME: Duration = Duration::from_millis(500);

#[derive(Resource)]
pub struct ConfigManager {
    _watcher: notify::RecommendedWatcher,
    rx: Receiver<()>,
    path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager that watches the given path for changes.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let (tx, rx) = flume::bounded(1);

        // create a file watcher
        let mut watcher = notify::recommended_watcher({
            let mut last_reload = Instant::now();

            move |e: Result<Event, notify::Error>| {
                if let Ok(event) = e
                    && matches!(event.kind, EventKind::Modify(_))
                {
                    let now = Instant::now();
                    if now.duration_since(last_reload) > DEBOUNCE_TIME {
                        tx.send(()).ok();
                        last_reload = now;
                    }
                }
            }
        })?;
        let path = path.as_ref().to_path_buf();
        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
            path,
        })
    }

    /// Check if the configuration file has changed and reload it if necessary.
    pub fn reload(manager: Res<Self>, mut config: ResMut<Config>) {
        if manager.rx.try_recv().is_ok() {
            // reload config
            match Config::load(&manager.path) {
                Ok(new_config) => *config = new_config,
                Err(e) => eprintln!("Error: {:?}", Report::new_boxed(Box::new(e))),
            }
        }
    }
}
