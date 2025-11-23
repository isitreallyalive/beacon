use std::{fs, ops::Deref, path::PathBuf, rc::Rc};

use notify::Watcher;
use serde::Deserialize;
use tokio::sync::{Mutex, mpsc};

pub use crate::def::ConfigData;
use crate::def::*;

#[macro_use]
extern crate tracing;

mod def;

#[derive(Debug, Default)]
pub struct Config {
    pub data: Rc<Mutex<ConfigData>>,
    watcher: Option<ConfigWatcher>,
    path: PathBuf,
}

impl Deref for Config {
    type Target = Mutex<ConfigData>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug)]
struct ConfigWatcher {
    _inner: notify::RecommendedWatcher,
    rx: mpsc::Receiver<notify::Event>,
}

impl Deref for ConfigWatcher {
    type Target = mpsc::Receiver<notify::Event>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

macro_rules! reload {
    ($config:expr, $value:expr, $log:expr, $field:ident, $struct:ty) => {
        if let Some(value) = $value.get(stringify!($field)) {
            match <$struct>::deserialize(value.clone()) {
                Ok($field) => {
                    if $config.$field == $field {
                        // no changes
                        return;
                    }

                    $config.$field = $field;
                    if $log {
                        info!("reloaded {} config", stringify!($field));
                    }
                }
                Err(e) => {
                    error!(
                        "failed to deserialize {} config: {}. falling back to default.",
                        stringify!($field),
                        e
                    );
                }
            }
        }
    };
}

impl Config {
    /// Read configuration from a file
    pub async fn read(path: PathBuf) -> Self {
        // attempt to create a file watcher
        let (tx, rx) = mpsc::channel(1);
        let watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .and_then(|mut watcher| {
            watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;
            Ok(watcher)
        })
        .map(|inner| ConfigWatcher { _inner: inner, rx })
        .ok();

        if let None = watcher {
            warn!("file watcher could not be created. config changes will not be detected.");
        }

        let mut config = Self {
            data: Rc::new(Mutex::new(ConfigData::default())),
            watcher,
            path,
        };
        config.reload(false).await;

        config
    }

    pub async fn tick(&mut self) {
        // handle file change events
        if let Some(watcher) = self.watcher.as_mut() {
            let mut should_reload = false;

            // check for at least one modify event
            while let Ok(event) = watcher.rx.try_recv() {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    should_reload = true;
                }
                // drain all events in the channel
            }

            if should_reload {
                self.reload(true).await;
            }
        }
    }

    /// Reload configuration from a file
    async fn reload(&mut self, log: bool) {
        let mut config = self.data.lock().await;

        let value = match fs::read_to_string(&self.path) {
            // try to read existing config file
            Ok(contents) => match toml::from_str::<toml::Value>(&contents) {
                Ok(v) => v,
                Err(e) => {
                    error!("failed to parse config: {}", e);
                    return;
                }
            },
            Err(_) => {
                // if the file doesn't exist, create one
                if let Err(e) =
                    toml::to_string_pretty(&*config).map(|contents| fs::write(&self.path, contents))
                {
                    error!("failed to write default config: {}", e);
                }
                return;
            }
        };

        reload!(config, value, log, server, ServerConfig);
        reload!(config, value, log, world, WorldConfig);
        reload!(config, value, log, query, QueryConfig);
    }
}
