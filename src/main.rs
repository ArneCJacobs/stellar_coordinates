use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{primitives::{Aabb, Sphere}, renderer::RenderDevice}, window::PresentMode,
};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use flate2::read::GzDecoder;
use itertools::Itertools;
use serde::Deserialize;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

use crate::chunk::Catalog;
use crate::gpu_instancing::{CustomMaterialPlugin, InstanceData};

mod chunk;
mod cursor;
mod gpu_instancing;
mod util;

struct StarsLOD(Vec<(u32, Handle<Mesh>)>);

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
        .insert_resource(StarsLOD(vec![]))
        .add_system(draw_bounding_box_system)
        .add_system(catalog_system)
        .add_startup_system(setup)
        .run();
}

fn draw_bounding_box_system(
    mut lines: ResMut<DebugLines>,
    query: Query<&bevy::render::primitives::Aabb>,
) {
    for aabb in query.iter() {
        util::draw_bounding_box(&mut lines, aabb);
    }
}

//fn LOD_system(
//mut query: Query<(Entity, &mut Handle<Mesh>, &ChunkPos),With<InstanceBuffer>>,
//camera: Query<(&Transform,), With<FpsCameraController>>,
//LOD_map: Res<StarsLOD>,
//commands: Commands,
//) {
//let (camera_transform,) = camera.get_single().unwrap();
//let camera_chunk_pos = to_chunk_location(camera_transform.translation);

//for (entity, mesh, chunk_pos) in query.iter_mut() {
//let diff = (camera_chunk_pos - chunk_pos.0).abs();
//let taxi_dist = (diff.x + diff.y + diff.z) as u32;
//let mut new_mesh = LOD_map.0[0].1;
//for (dist, mesh_handle) in LOD_map.0.into_iter() {
//if taxi_dist <= dist {
//new_mesh = mesh_handle;
//} else {
//break;
//}
//}
//commands.entity(entity).insert(new_mesh);
//}

//}


#[derive(Deserialize)]
struct Pos {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct ChunkPos(IVec3);

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

    let view_radius = ViewRadiusResource{ radius: 4.0 };
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
    //LOD_map.0.push((0, mesh_close));
    //LOD_map.0.push((1, mesh_far));
}
