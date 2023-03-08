mod math;

use bevy::prelude::*;

pub mod prelude {
    pub use super::{Orbit, OrbitPlugin};
}

pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            calculate_orbits.before(bevy::transform::transform_propagate_system),
        );
    }
}

#[derive(Component)]
pub struct Orbit {
    pub semi_major_axis: f32,
    pub eccentricity: f32,
    pub argument_of_periapsis: f32,
    pub initial_mean_anomaly: f32,
}

pub fn calculate_orbits(time: Res<Time>, mut query: Query<(&Orbit, &mut Transform)>) {
    for (orbit, mut transform) in query.iter_mut() {
        if orbit.semi_major_axis == 0.0 {
            transform.translation = Vec3::ZERO;
        } else {
            let pos = math::calculate_position_at_time(
                orbit.semi_major_axis,
                orbit.eccentricity,
                orbit.argument_of_periapsis,
                orbit.initial_mean_anomaly,
                1_000_000_000_000.0,
                time.elapsed_seconds(),
            );
            transform.translation = Vec3::from(pos);
        }
    }
}
