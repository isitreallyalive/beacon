use beacon_config::{Config, ConfigUpdate};
use beacon_query::QueryActor;
use kameo::prelude::*;
use kameo_actors::{
    pubsub::{PubSub, Subscribe},
    scheduler::Scheduler,
};

use crate::BeaconError;

pub struct BeaconActor {
    config: Conig

    scheduler: ActorRef<Scheduler>,
    query: Option<ActorRef<QueryActor>>,
}

impl Actor for BeaconActor {
    type Args = (Config, ActorRef<PubSub<ConfigUpdate>>);
    type Error = BeaconError;

    async fn on_start(
        (config, config_update): Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let scheduler = Scheduler::spawn_default();

        // subscribe for config updates
        config_update.tell(Subscribe(actor_ref)).await?;

        let mut actor = Self {
            scheduler,
            config_update,
            query: None,
        };
        actor.sync_actors(config).await?;

        Ok(actor)
    }
}

impl BeaconActor {
    pub async fn sync_actors(&mut self, config: Config) -> Result<(), BeaconError> {
        // self.query = if config.query.enable && self.query.is_none() {
        //     // query should be running
        //     let query = QueryActor::spawn((config.clone(), self.scheduler.clone()));
        //     self.config_update.tell(Subscribe(query.clone())).await?;
        //     Some(query)
        // } else {
        // };

        if let Some(query) = self.query {
            if !config.query.enable {
                // spawn query
            }
        }

        Ok(())
    }
}

impl Message<ConfigUpdate> for BeaconActor {
    type Reply = ();

    async fn handle(
        &mut self,
        ConfigUpdate(config): ConfigUpdate,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let _ = self.sync_actors(config).await;
    }
}
