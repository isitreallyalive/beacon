use std::path::PathBuf;

use beacon::BeaconError;
use beacon_query::QueryActor;
use kameo::prelude::*;
use kameo_actors::scheduler::Scheduler;

use crate::server::config::BeaconConfig;

mod config;

/// Supervisor actor for the beacon server.
pub(crate) struct BeaconActor {
    config: BeaconConfig,

    /// Scheduler for managing tasks on an interval.
    scheduler: ActorRef<Scheduler>,
    query: Option<ActorRef<QueryActor>>,
}

impl Actor for BeaconActor {
    type Args = PathBuf;
    type Error = BeaconError;

    async fn on_start(
        config_path: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let config = BeaconConfig::new(config_path, actor_ref)?;
        let scheduler = Scheduler::spawn_default();

        // sync with config
        let mut actor = Self {
            scheduler,
            config,
            query: None,
        };
        actor.sync();

        Ok(actor)
    }
}

impl BeaconActor {
    fn sync(&mut self) {
        match &self.query {
            Some(query) if !self.config.query.enable => {
                query.kill();
                self.query = None;
            }
            None if self.config.query.enable => {
                self.query = Some(QueryActor::spawn((
                    self.config.clone(),
                    self.scheduler.clone(),
                )));
            }
            _ => {}
        }
    }
}
