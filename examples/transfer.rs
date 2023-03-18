mod utils;

use bevy::prelude::*;
use bevy_mod_orbits::prelude::*;
use bevy_polyline::prelude::*;
use utils::draw_ellipse;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PolylinePlugin)
        .add_plugin(OrbitPlugin)
        .add_startup_system(startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ..default()
    });

    let mesh = meshes.add(
        shape::Icosphere {
            radius: 0.2,
            subdivisions: 32,
        }
        .into(),
    );

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.7, 0.3, 0.3),
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

    // Visualise the initial and target orbit
    let mut initial_polyline = Polyline::default();
    draw_ellipse(&initial_orbit, &mut initial_polyline);
    commands.spawn(PolylineBundle {
        polyline: polylines.add(initial_polyline),
        material: polyline_materials.add(PolylineMaterial {
            width: 1.0,
            color: Color::WHITE,
            ..default()
        }),
        ..default()
    });

    let mut target_polyline = Polyline::default();
    draw_ellipse(&target_orbit, &mut target_polyline);
    commands.spawn(PolylineBundle {
        polyline: polylines.add(target_polyline),
        material: polyline_materials.add(PolylineMaterial {
            width: 1.0,
            color: Color::WHITE,
            ..default()
        }),
        ..default()
    });

    let mut transfer_polyline = Polyline::default();
    draw_ellipse(&transfer_orbit, &mut transfer_polyline);
    commands.spawn(PolylineBundle {
        polyline: polylines.add(transfer_polyline),
        material: polyline_materials.add(PolylineMaterial {
            width: 1.0,
            color: Color::WHITE,
            ..default()
        }),
        ..default()
    });
}
