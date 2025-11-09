use bevy_ecs::{component::Mutable, prelude::*};

use crate::Listener;

pub trait Connection: Component<Mutability = Mutable> + Sized {
    type Listener: Listener;

    fn register(schedule: &mut Schedule) {
        schedule.add_systems(Self::process);
    }

    /// System to process existing connections.
    fn process(
        connections: Query<(Entity, &mut Self)>,
        listener: Option<Res<Self::Listener>>,
        commands: Commands,
    ) -> Result<()>;
}
