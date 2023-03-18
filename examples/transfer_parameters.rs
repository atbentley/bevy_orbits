mod utils;

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_orbits::prelude::*;
use bevy_polyline::prelude::*;
use utils::draw_ellipse;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PolylinePlugin)
        .add_plugin(OrbitPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(startup)
        .add_system(ui)
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

    commands
        .spawn((
            PbrBundle {
                mesh,
                material,
                ..default()
            },
            initial_orbit.clone(),
            TransferSchedule::default(),
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
}

struct NextTransfer {
    execution_time: f32,
    transfer: Transfer,
    transfer_entity: Entity,
    target_entity: Entity,
    done: bool,
}

impl FromWorld for NextTransfer {
    fn from_world(world: &mut World) -> NextTransfer {
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

        let transfer = calculate_hohmann_transfer(&initial_orbit, &target_orbit, 1e11, 5.0);
        let transfer_orbit = transfer.maneuvers.front().unwrap().target_orbit.clone();
        let target_orbit = transfer.maneuvers.back().unwrap().target_orbit.clone();

        let mut polylines = world.resource_mut::<Assets<Polyline>>();
        let mut transfer_polyline = Polyline::default();
        draw_ellipse(&transfer_orbit, &mut transfer_polyline);
        let transfer_polyline_handle = polylines.add(transfer_polyline);

        let mut target_polyline = Polyline::default();
        draw_ellipse(&target_orbit, &mut target_polyline);
        let target_polyline_handle = polylines.add(target_polyline);

        let mut polyline_materials = world.resource_mut::<Assets<PolylineMaterial>>();
        let material_handle = polyline_materials.add(PolylineMaterial {
            width: 1.0,
            color: Color::WHITE,
            ..default()
        });

        let transfer_entity = world
            .spawn(PolylineBundle {
                polyline: transfer_polyline_handle,
                material: material_handle.clone(),
                ..default()
            })
            .id();
        let target_entity = world
            .spawn(PolylineBundle {
                polyline: target_polyline_handle,
                material: material_handle,
                ..default()
            })
            .id();
        NextTransfer {
            execution_time: 5.0,
            transfer,
            transfer_entity,
            target_entity,
            done: false,
        }
    }
}

fn ui(
    time: Res<Time>,
    mut state: Local<NextTransfer>,
    mut egui_context: ResMut<EguiContext>,
    polyline_handles: Query<&Handle<Polyline>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut query: Query<(&Orbit, &mut TransferSchedule)>,
) {
    let draw_orbit = |ui: &mut egui::Ui, orbit: &mut Orbit| {
        ui.label("Semi-major axis");
        ui.add(egui::Slider::new(&mut orbit.semi_major_axis, 0.0..=5.0)).changed();

        ui.label("Eccentricity");
        ui.add(egui::Slider::new(&mut orbit.eccentricity, 0.0..=1.0)).changed();

        ui.label("Argument of periapsis");
        ui.add(egui::Slider::new(&mut orbit.argument_of_periapsis, 0.0..=TAU)).changed();

        ui.label("Initial mean anomaly");
        ui.add(egui::Slider::new(&mut orbit.initial_mean_anomaly, 0.0..=TAU)).changed();
    };

    let (initial_orbit, mut schedule) = query.single_mut();

    if !state.done {
        egui::Window::new("Transfer").show(egui_context.ctx_mut(), |ui| {
            ui.heading("Traget Orbit");
            let mut target_orbit = state.transfer.maneuvers.back().unwrap().target_orbit.clone();
            draw_orbit(ui, &mut target_orbit);
            ui.add_space(ui.spacing().item_spacing.y * 2.0);

            ui.heading("Transfer");
            ui.label("Execution time (seconds from now)");
            ui.add(egui::Slider::new(&mut state.execution_time, 0.0..=10.0)).changed();

            if let Ok(polyline_handle) = polyline_handles.get(state.target_entity) {
                if let Some(mut polyline) = polylines.get_mut(polyline_handle) {
                    draw_ellipse(&target_orbit, &mut polyline);
                }
            }

            let when = time.elapsed_seconds() + state.execution_time;
            let transfer = calculate_hohmann_transfer(&initial_orbit, &target_orbit, 1e11, when);
            let transfer_orbit = transfer.maneuvers.front().unwrap().target_orbit.clone();
            state.transfer = transfer.clone();

            if let Ok(polyline_handle) = polyline_handles.get(state.transfer_entity) {
                if let Some(mut polyline) = polylines.get_mut(polyline_handle) {
                    draw_ellipse(&transfer_orbit, &mut polyline);
                }
            }

            if ui.button("Go").clicked() {
                schedule.push_transfer(transfer);
                state.done = true;
            }
        });
    }
}
