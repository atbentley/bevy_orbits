use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_mod_orbits::prelude::*;
use bevy_polyline::polyline::*;

pub fn draw_ellipse(orbit: &Orbit, polyine: &mut Polyline) {
    polyine.vertices.clear();

    let semi_minor_axis = orbit.semi_major_axis * (1.0 - orbit.eccentricity.powi(2)).sqrt();
    let linear_eccentricity = (orbit.semi_major_axis.powi(2) - semi_minor_axis.powi(2)).sqrt();
    let segments = 64;

    let rotation = Quat::from_axis_angle(Vec3::Y, -orbit.argument_of_periapsis);
    for i in 0..(segments + 1) {
        let t = i as f32 * (TAU / segments as f32);
        let x = orbit.semi_major_axis * t.cos();
        let y = semi_minor_axis * t.sin();
        let point = Vec3::new(x - linear_eccentricity, 0.0, y);
        let rotated_point = rotation * point;

        polyine.vertices.push(rotated_point);
    }
}
