use bevy::prelude::*;
use bevy_orbits::prelude::*;

#[bevy_main]
fn main() {
    App::new().add_plugins((DefaultPlugins, OrbitPlugin)).add_systems(Startup, startup).run();
}

fn startup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ..default()
    });

    let mesh = meshes.add(Sphere::new(0.45));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.3, 0.3),
        unlit: true,
        ..default()
    });

    let sun = commands
        .spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                ..default()
            },
            Mass { mass: 1e12 },
        ))
        .id();

    commands
        .spawn((
            PbrBundle {
                mesh,
                material,
                ..default()
            },
            Orbit {
                semi_major_axis: 4.0,
                eccentricity: 0.0,
                argument_of_periapsis: 0.0,
                initial_mean_anomaly: 0.0,
            },
        ))
        .set_parent(sun);
}
