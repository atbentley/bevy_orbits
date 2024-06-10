use std::f32::consts::PI;

pub use bevy::prelude::*;
pub use bevy_mod_orbits::prelude::Orbit;

#[allow(unused)]
pub fn draw_orbit(gizmos: &mut Gizmos, orbit: &Orbit, parent_position: Vec3) {
    let initial_rotation = Quat::from_rotation_x(PI / 2.0);
    let argument_of_periapsis_rotation = Quat::from_rotation_y(-orbit.argument_of_periapsis);
    let final_rotation = argument_of_periapsis_rotation * initial_rotation;

    let half_size = Vec2::new(
        orbit.semi_major_axis,
        orbit.semi_major_axis * (1.0 - orbit.eccentricity.powi(2)).sqrt(),
    );

    let ellipse_center_offset = Vec3::new(-(half_size.x.powi(2) - half_size.y.powi(2)).sqrt(), 0.0, 0.0);
    let ellipse_center = parent_position + argument_of_periapsis_rotation * ellipse_center_offset;

    gizmos.ellipse(ellipse_center, final_rotation, half_size, Color::WHITE).segments(64);
}
