use bevy::prelude::*;

use crate::orbit::calculate_orbits;
use crate::transfer::execute_orbital_maneuvers;

pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(execute_orbital_maneuvers.before(bevy::transform::transform_propagate_system))
                .with_system(calculate_orbits.after(execute_orbital_maneuvers)),
        );
    }
}
