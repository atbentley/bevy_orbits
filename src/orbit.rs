use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Orbit {
    pub semi_major_axis: f32,
    pub eccentricity: f32,
    pub argument_of_periapsis: f32,
    pub initial_mean_anomaly: f32,
}

#[derive(Component)]
pub struct Mass {
    pub mass: f32,
}

pub fn calculate_orbits(
    time: Res<Time>,
    masses: Query<&Mass>,
    mut orbits: Query<(&Orbit, &mut Transform, Option<&Parent>)>,
) {
    for (orbit, mut transform, maybe_parent) in orbits.iter_mut() {
        if orbit.semi_major_axis == 0.0 {
            transform.translation = Vec3::ZERO;
            continue;
        }

        let Some(parent) = maybe_parent else {
            transform.translation = Vec3::ZERO;
            continue;
        };

        let Some(parent_mass) = masses.get_component::<Mass>(parent.get()).ok() else {
            warn!("Parent entity {parent:?} is missing Mass component");
            continue;
        };

        let pos = crate::math::calculate_position_at_time(
            orbit.semi_major_axis,
            orbit.eccentricity,
            orbit.argument_of_periapsis,
            orbit.initial_mean_anomaly,
            parent_mass.mass,
            time.elapsed_seconds(),
        );
        transform.translation = Vec3::from(pos);
    }
}
