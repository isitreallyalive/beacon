use std::{fs, io, ops::Deref, path::PathBuf, sync::Arc};

use beacon_util::{Tickable, async_trait};
use notify::Watcher;
use serde::Deserialize;
use tokio::sync::{Mutex, broadcast, mpsc};

pub use crate::def::ConfigData;
use crate::def::*;

#[macro_use]
extern crate tracing;

mod def;

#[derive(Debug, Default)]
pub struct Config {
    /// The actual configuration data
    pub data: Arc<Mutex<ConfigData>>,
    /// File watcher for config changes
    watcher: Option<ConfigWatcher>,
    /// Path to the configuration file
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
    /// The actual file watcher
    _inner: notify::RecommendedWatcher,
    /// Receiver for file change events
    rx: mpsc::Receiver<notify::Event>,
    tx: broadcast::Sender<()>,
}

impl Deref for ConfigWatcher {
    type Target = mpsc::Receiver<notify::Event>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

macro_rules! reload {
    ($config:expr, $watcher:expr, $value:expr, $log:expr, $announced:expr, $field:ident, $struct:ty) => {
        if let Some(value) = $value.get(stringify!($field)) {
            match <$struct>::deserialize(value.clone()) {
                Ok($field) => {
                    if $config.$field == $field {
                        // no changes
                        return;
                    } else if !*$announced {
                        let _ = $watcher.as_ref().map(|w| w.tx.send(()));
                        *$announced = true;
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

#[async_trait]
impl Tickable for Config {
    async fn tick(&mut self) -> io::Result<()> {
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

        Ok(())
    }
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
        .map(|inner| ConfigWatcher {
            _inner: inner,
            rx,
            tx: broadcast::channel(1).0,
        })
        .ok();

        if let None = watcher {
            warn!("file watcher could not be created. config changes will not be detected.");
        }

        let mut config = Self {
            data: Arc::new(Mutex::new(ConfigData::default())),
            watcher,
            path,
        };
        config.reload(false).await;

        config
    }

    /// Subscribe to configuration change events
    pub async fn subscribe(&self) -> Option<broadcast::Receiver<()>> {
        self.watcher.as_ref().map(|w| w.tx.subscribe())
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

        let mut announced = false;
        reload!(
            config,
            self.watcher,
            value,
            log,
            &mut announced,
            server,
            ServerConfig
        );
        reload!(
            config,
            self.watcher,
            value,
            log,
            &mut announced,
            world,
            WorldConfig
        );
        reload!(
            config,
            self.watcher,
            value,
            log,
            &mut announced,
            query,
            QueryConfig
        );
    }
}
