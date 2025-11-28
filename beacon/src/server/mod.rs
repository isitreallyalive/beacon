use std::{env::consts, path::PathBuf};

use beacon::BeaconError;
use beacon_data::BEACON_VERSION;
use beacon_java::JavaActor;
use beacon_query::QueryActor;
use beacon_tui::{Stop, TuiActor};
use kameo::prelude::*;
use kameo_actors::scheduler::Scheduler;

use crate::server::config::BeaconConfig;

mod config;

/// Supervisor actor for the beacon server.
pub(crate) struct BeaconActor {
    config: BeaconConfig,
    scheduler: ActorRef<Scheduler>,
    #[allow(dead_code)]
    tui: ActorRef<TuiActor<Self>>,
    java: ActorRef<JavaActor>,
    query: Option<ActorRef<QueryActor>>,
}

impl Actor for BeaconActor {
    type Args = PathBuf;
    type Error = BeaconError;

    async fn on_start(
        config_path: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let config = BeaconConfig::new(config_path, actor_ref.clone())?;
        let scheduler = Scheduler::spawn_default();
        let tui = TuiActor::spawn_with_mailbox(actor_ref, mailbox::unbounded());
        let java = JavaActor::spawn(config.clone());

        // sync with config
        let mut actor = Self {
            config,
            scheduler,
            tui,
            java,
            query: None,
        };
        actor.sync();

        info!("starting beacon v{BEACON_VERSION}");
        warn!("beacon is in early development. expect bugs and incomplete features.");
        debug!(
            family = consts::FAMILY,
            os = consts::OS,
            arch = consts::ARCH,
            protocol = beacon_data::PROTOCOL_VERSION,
            supports = ?beacon_data::SUPPORTED_VERSIONS,
            debug = cfg!(debug_assertions),
            "build info"
        );

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
                    self.java.clone(),
                    self.scheduler.clone(),
                )));
            }
            _ => {}
        }
    }
}

impl Message<Stop> for BeaconActor {
    type Reply = ();

    async fn handle(&mut self, _msg: Stop, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        info!("shutting down beacon");
        // todo: gracefully stop other actors
        ctx.stop();
    }
}
