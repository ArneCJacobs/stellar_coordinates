use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{primitives::Aabb, renderer::RenderDevice}, window::PresentMode,
};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

use crate::chunk::Catalog;
use crate::gpu_instancing::CustomMaterialPlugin;

mod chunk;
mod cursor;
mod gpu_instancing;
mod util;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new()) // in game inspector
        .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            // FrameTimeDiagnosticsPlugin::FPS,
        ]))
        .add_plugin(DebugLinesPlugin::default())
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            present_mode: PresentMode::Immediate,
            ..default()
        })
        // .add_system(draw_bounding_box_system)
        .add_system(catalog_system)
        .add_startup_system(setup)
        .run();
}

#[allow(dead_code)]
fn draw_bounding_box_system(
    mut lines: ResMut<DebugLines>,
    query: Query<&bevy::render::primitives::Aabb>,
) {
    for aabb in query.iter() {
        util::draw_bounding_box(&mut lines, aabb);
    }
}

#[derive(Component)]
struct Player();

fn catalog_system(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut catalog: ResMut<Catalog>,
    view_radius: Res<ViewRadiusResource>,
    ) {
    let player_transform = player_query.get_single_mut().unwrap();
    catalog.particle_loader.update_chunks(&mut commands, render_device, player_transform.translation, view_radius.radius);
     
}

#[derive(Clone, Copy)]
struct ViewRadiusResource {
    radius: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    render_device: Res<RenderDevice>,
) {
    let ico_sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.1f32,
        subdivisions: 0,
    }));

    let view_radius = ViewRadiusResource{ radius: 40.0 };
    commands.insert_resource(view_radius);
    let mut catalog = Catalog::new(
        "catalog_gaia_dr3_extralarge".to_string(),
        ico_sphere,
    );
    catalog.particle_loader.update_chunks(&mut commands, render_device, Vec3::ZERO, view_radius.radius);
    commands.insert_resource(catalog);

    let controller = FpsCameraController {
        smoothing_weight: 0.6,
        translate_sensitivity: 0.1,
        mouse_rotate_sensitivity: Vec2::splat(0.001),
        ..Default::default()
    };
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert(Player())
        .insert_bundle(FpsCameraBundle::new(
            controller,
            PerspectiveCameraBundle::default(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1., 0., 0.),
        ));
}
