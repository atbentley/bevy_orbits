mod utils;

use bevy::prelude::*;
use bevy_orbits::prelude::*;
use utils::draw_orbit;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, OrbitPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, draw_orbits)
        .run();
}

#[derive(Resource)]
pub struct Orbits {
    start: Orbit,
    transfer: Orbit,
    target: Orbit,
}

fn startup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ..default()
    });

    let mesh = meshes.add(Sphere::new(0.2));

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
            Mass { mass: 1e11 },
        ))
        .id();

    let initial_orbit = Orbit {
        semi_major_axis: 2.0,
        eccentricity: 0.0,
        argument_of_periapsis: 0.0,
        initial_mean_anomaly: 0.0,
    };

    let target_orbit = Orbit {
        semi_major_axis: 4.0,
        eccentricity: 0.0,
        argument_of_periapsis: 0.0,
        initial_mean_anomaly: 0.0,
    };

    let transfer = calculate_hohmann_transfer(&initial_orbit, &target_orbit, 1e11, 2.0);
    let transfer_orbit = transfer.maneuvers.front().unwrap().target_orbit.clone();
    let mut schedule = TransferSchedule::default();
    schedule.push_transfer(transfer);

    commands
        .spawn((
            PbrBundle {
                mesh,
                material,
                ..default()
            },
            initial_orbit.clone(),
            schedule,
        ))
        .set_parent(sun);

    let orbits = Orbits {
        start: initial_orbit,
        transfer: transfer_orbit,
        target: target_orbit,
    };
    commands.insert_resource(orbits);
}

pub fn draw_orbits(mut gizmos: Gizmos, orbits: Res<Orbits>) {
    draw_orbit(&mut gizmos, &orbits.start, Vec3::ZERO);
    draw_orbit(&mut gizmos, &orbits.transfer, Vec3::ZERO);
    draw_orbit(&mut gizmos, &orbits.target, Vec3::ZERO);
}
