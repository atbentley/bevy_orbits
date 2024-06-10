mod utils;

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_egui::egui::Ui;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_orbits::prelude::*;
use format_num::format_num;

use utils::{dejitter_orbit, draw_orbit, MassChanged, OrbitChanged};

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, OrbitPlugin, EguiPlugin))
        .add_event::<MassChanged>()
        .add_event::<OrbitChanged>()
        .add_systems(Startup, startup)
        .add_systems(Update, (ui, dejitter_orbit, draw_orbits).chain())
        .run();
}

fn startup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, -Vec3::Z),
        ..default()
    });

    let sun = commands
        .spawn((
            Sun,
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.45)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.7, 0.3, 0.3),
                    unlit: true,
                    ..default()
                }),
                ..default()
            },
            Mass { mass: 1e12 },
        ))
        .id();

    let earth = commands
        .spawn((
            Earth,
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.2)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.7, 0.3, 0.3),
                    unlit: true,
                    ..default()
                }),
                ..default()
            },
            Orbit {
                semi_major_axis: 4.0,
                eccentricity: 0.0,
                argument_of_periapsis: 0.0,
                initial_mean_anomaly: 0.0,
            },
            Mass { mass: 1e10 },
        ))
        .set_parent(sun)
        .id();

    commands
        .spawn((
            Moon,
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.1)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.7, 0.3, 0.3),
                    unlit: true,
                    ..default()
                }),
                ..default()
            },
            Orbit {
                semi_major_axis: 1.0,
                eccentricity: 0.0,
                argument_of_periapsis: 0.0,
                initial_mean_anomaly: 0.0,
            },
        ))
        .set_parent(earth);
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct Earth;

#[derive(Component)]
struct Moon;

fn ui(
    mut egui_contexts: EguiContexts,
    mut queries: ParamSet<(
        Query<(Entity, &mut Mass), With<Sun>>,
        Query<(Entity, &mut Orbit, &mut Mass), With<Earth>>,
        Query<(Entity, &mut Orbit), With<Moon>>,
    )>,
    mut mass_changed: EventWriter<MassChanged>,
    mut orbit_changed: EventWriter<OrbitChanged>,
) {
    let mut draw_mass = |ui: &mut Ui, entity: Entity, mass: &mut Mut<Mass>| {
        let inner_mass = mass.bypass_change_detection();
        let original_mass = inner_mass.mass;

        ui.label("Mass");
        let mass_slider = egui::Slider::new(&mut inner_mass.mass, 0.0..=1e20)
            .logarithmic(true)
            .custom_formatter(|x, _| format!("{}g", format_num!(".0s", x)));
        if ui.add(mass_slider).changed() {
            mass_changed.send(MassChanged {
                entity: entity,
                old_mass: original_mass,
                new_mass: mass.mass,
            });
            mass.set_changed();
        }
    };

    let mut draw_orbit = |ui: &mut Ui, entity: Entity, orbit: &mut Mut<Orbit>| {
        let inner_orbit = orbit.bypass_change_detection();
        let original_orbit = inner_orbit.clone();
        let mut changed = false;

        ui.label("Semi-major axis");
        changed |= ui.add(egui::Slider::new(&mut orbit.semi_major_axis, 0.0..=5.0)).changed();

        ui.label("Eccentricity");
        changed |= ui.add(egui::Slider::new(&mut orbit.eccentricity, 0.0..=1.0)).changed();

        ui.label("Argument of periapsis");
        changed |= ui.add(egui::Slider::new(&mut orbit.argument_of_periapsis, 0.0..=TAU)).changed();

        ui.label("Initial mean anomaly");
        changed |= ui.add(egui::Slider::new(&mut orbit.initial_mean_anomaly, 0.0..=TAU)).changed();

        if changed {
            orbit_changed.send(OrbitChanged {
                entity: entity,
                old_orbit: original_orbit,
                new_orbit: orbit.clone(),
            });
            orbit.set_changed();
        }
    };

    egui::Window::new("Parameters").resizable(false).show(egui_contexts.ctx_mut(), |ui| {
        ui.heading("Sun");
        let mut suns = queries.p0();
        let (entity, mut mass) = suns.single_mut();
        draw_mass(ui, entity, &mut mass);
        ui.add_space(ui.spacing().item_spacing.y * 2.0);

        ui.heading("Earth");
        let mut earths = queries.p1();
        let (entity, mut orbit, mut mass) = earths.single_mut();
        draw_mass(ui, entity, &mut mass);
        draw_orbit(ui, entity, &mut orbit);
        ui.add_space(ui.spacing().item_spacing.y * 2.0);

        ui.heading("Moon");
        let mut moons = queries.p2();
        let (entity, mut orbit) = moons.single_mut();
        draw_orbit(ui, entity, &mut orbit);
    });
}

pub fn draw_orbits(mut gizmos: Gizmos, orbits: Query<(&Orbit, &GlobalTransform, Option<&Parent>)>) {
    for (orbit, _, maybe_parent) in &orbits {
        let parent_position = maybe_parent
            .and_then(|parent| orbits.get(parent.get()).ok())
            .map(|(_, parent_transform, _)| parent_transform.translation())
            .unwrap_or(Vec3::ZERO);

        draw_orbit(&mut gizmos, &orbit, parent_position);
    }
}
