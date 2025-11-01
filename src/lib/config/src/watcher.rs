use std::time::Duration;

use bevy_ecs::prelude::*;
use notify::Watcher;

use crate::{Config, ConfigError};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

#[derive(Resource)]
pub(crate) struct ConfigWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: crossbeam_channel::Receiver<notify::Result<notify::Event>>,
}

impl ConfigWatcher {
    pub(crate) fn setup(
        world: &mut World,
        schedule: &mut Schedule,
        path: &std::path::Path,
    ) -> Result<(), ConfigError> {
        // init
        let (tx, rx) = crossbeam_channel::bounded(1);
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(path, notify::RecursiveMode::NonRecursive)?;

        // register
        world.insert_resource(ConfigWatcher {
            _watcher: watcher,
            rx,
        });
        schedule.add_systems(ConfigWatcher::watch);
        Ok(())
    }

    fn watch(watcher: Res<ConfigWatcher>, mut config_res: ResMut<Config>) -> Result<()> {
        while let Ok(event) = watcher.rx.try_recv() {
            match event {
                Ok(event) => {
                    if !matches!(event.kind, notify::EventKind::Modify(_)) {
                        continue;
                    }
                    // make sure the write is done
                    std::thread::sleep(DEBOUNCE_DURATION);
                    let path = event
                        .paths
                        .first()
                        .expect("there should always be a path associated with notify::Event");
                    match Config::read(path) {
                        Ok(config) => {
                            if *config_res != config {
                                info!("config changed: {:?}", config);
                                *config_res = config;
                            }
                        }
                        Err(e) => {
                            error!("failed to read updated config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("watch error: {:?}", e);
                }
            }
        }
        Ok(())
    }
}
