use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_mod_orbits::math::*;
use bevy_mod_orbits::prelude::*;

#[allow(dead_code)]
pub struct OrbitChanged {
    pub entity: Entity,
    pub old_orbit: Orbit,
    pub new_orbit: Orbit,
}

#[allow(dead_code)]
pub struct MassChanged {
    pub entity: Entity,
    pub old_mass: f32,
    pub new_mass: f32,
}

// If an orbit's semi-major axis is changed, or if it's parent's mass is changed, then
// the body will move significantly forwards or backwards through it's orbital path.
// This function recalculates an orbit's initial mean anomaly to give a smoother appearance.
#[allow(dead_code)]
pub fn dejitter_orbit(
    mut changed_masses: EventReader<MassChanged>,
    mut changed_orbits: EventReader<OrbitChanged>,
    masses: Query<&Mass>,
    mut orbits: Query<&mut Orbit>,
    parents: Query<&Parent>,
    childrens: Query<&Children>,
    time: Res<Time>,
) {
    for change in changed_masses.iter() {
        let Some(children) = childrens.get(change.entity).ok() else { continue; };
        for child_entity in children {
            let Some(mut orbit) = orbits.get_mut(*child_entity).ok() else { continue; };
            let new_initial_mean_anomaly = calculate_initial_mean_anomaly(
                orbit.semi_major_axis,
                orbit.initial_mean_anomaly,
                change.old_mass,
                orbit.semi_major_axis,
                orbit.eccentricity,
                change.new_mass,
                time.elapsed_seconds(),
            );
            orbit.initial_mean_anomaly = new_initial_mean_anomaly;
        }
    }

    for change in changed_orbits.iter() {
        let Some(parent_entity) = parents.get(change.entity).ok() else { continue; };
        let Some(parent_mass) = masses.get(parent_entity.get()).ok() else { continue; };
        let Some(mut orbit) = orbits.get_mut(change.entity).ok() else { continue; };
        let new_initial_mean_anomaly = calculate_initial_mean_anomaly(
            change.old_orbit.semi_major_axis,
            change.old_orbit.initial_mean_anomaly,
            parent_mass.mass,
            change.new_orbit.semi_major_axis,
            change.new_orbit.eccentricity,
            parent_mass.mass,
            time.elapsed_seconds(),
        );
        orbit.initial_mean_anomaly = new_initial_mean_anomaly;
    }
}

fn calculate_initial_mean_anomaly(
    original_semi_major_axis: f32,
    original_initial_mean_anomaly: f32,
    original_parent_mass: f32,
    semi_major_axis: f32,
    eccentricity: f32,
    parent_mass: f32,
    time: f32,
) -> f32 {
    let original_period = calculate_period(original_semi_major_axis, original_parent_mass);
    let original_mean_motion = calculate_mean_motion(original_period);
    let original_mean_anomaly = calculate_mean_anomaly(original_mean_motion, original_initial_mean_anomaly, time);
    let original_eccentric_anomaly = calculate_eccentric_anomaly(eccentricity, original_mean_anomaly);
    let original_true_anomaly = calculate_true_anomaly(eccentricity, original_eccentric_anomaly);

    let e = eccentricity;

    let f = |ea: f32| 2.0 * (((1.0 + e) / (1.0 - e)) * (ea / 2.0).tan().powi(2)).sqrt().atan() - original_true_anomaly;
    let df = |ea: f32| {
        ((1.0 + e) * (ea / 2.0).tan() * (1.0 / ((ea / 2.0).cos())).powi(2))
            / ((1.0 - e)
                * (((1.0 + e) * (ea / 2.0).tan().powi(2)) / (1.0 - e)).sqrt()
                * (((1.0 + e) * (ea / 2.0).tan().powi(2)) / (1.0 - e) + 1.0))
    };
    let mut ea = original_eccentric_anomaly;
    for _i in 0..15 {
        ea = ea - f(ea) / df(ea);
    }

    let mean_anomaly = ea - e * ea.sin();
    let mean_motion = calculate_mean_motion(calculate_period(semi_major_axis, parent_mass));
    (mean_anomaly - mean_motion * time).rem_euclid(TAU)
}
