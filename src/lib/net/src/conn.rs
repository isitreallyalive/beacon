use bevy_ecs::{component::Mutable, prelude::*};

pub trait Connection: Component<Mutability = Mutable> + Sized {
    fn register(schedule: &mut Schedule) {
        schedule.add_systems(Self::process);
    }

    /// System to process existing connections.
    fn process(connections: Query<(Entity, &mut Self)>, commands: Commands);
}
