use bevy::prelude::*;

use crate::orbit::calculate_orbits;
use crate::transfer::execute_orbital_maneuvers;

pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                execute_orbital_maneuvers.before(bevy::transform::systems::propagate_transforms),
                calculate_orbits.after(execute_orbital_maneuvers),
            ),
        );
    }
}
