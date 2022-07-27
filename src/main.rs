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

use crate::chunk::util::METADATA_FILE;
use crate::chunk::Catalog;
use crate::gpu_instancing::{CustomMaterialPlugin, InstanceData};

mod chunk;
mod cursor;
mod gpu_instancing;
mod util;

const CHUNK_SIZE: f32 = 50.0;
const LIMIT: u32 = 3_000_000;
const SCALE: f32 = 2.0;
const STAR_COUNT: u32 = 3_000_000;

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
            FrameTimeDiagnosticsPlugin::FPS,
        ]))
        .add_plugin(DebugLinesPlugin::default())
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            // present_mode: PresentMode::Immediate,
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

fn load_chunks() -> HashMap<IVec3, Vec<InstanceData>> {
    let file = File::open("./data/stars_big_transformed.csv.gz").expect("Could not open file");
    let decoder = GzDecoder::new(file);
    let reader = csv::ReaderBuilder::new().from_reader(decoder);

    let mut index = 0;
    let mut stars: Vec<InstanceData> = vec![];
    for record in reader.into_deserialize::<Pos>() {
        if let Ok(star_pos) = record {
            print!("\r                                            ");
            print!(
                "\r {:06}/{} {:.3}%",
                index,
                STAR_COUNT,
                (index as f32) / (STAR_COUNT as f32) * 100f32
            );

            let star_pos_inst = InstanceData {
                position: Vec3::new(star_pos.x * SCALE, star_pos.z * SCALE, star_pos.y * SCALE),
                scale: 1.0,
                color: Color::hex("ffd891").unwrap().as_rgba_f32(),
            };
            stars.push(star_pos_inst);
        }
        //let star_pos: Pos = record.unwrap();
        index += 1;
        if index >= LIMIT {
            break;
        }
    }
    //let max_radius = stars.iter()
    //.map(|star: &InstanceData| (star.position.x.powf(2.0) + star.position.y.powf(2.0) + star.position.z.powf(2.0)).powf(0.5))
    //.fold(0.0f32, |num, acc| num.max(acc));

    //let temp_stars = stars.iter().map(|star| {
    //InstanceData {
    //position: star.position * max_radius,
    //scale: 1.0,
    //color:  Color::hex("91ffd8").unwrap().as_rgba_f32(),
    //}
    //}).collect::<Vec<InstanceData>>();

    return stars
        .into_iter()
        .into_group_map_by(|star_pos| to_chunk_location(star_pos.position));
}

fn to_chunk_location(location: Vec3) -> IVec3 {
    (location / CHUNK_SIZE).floor().as_ivec3()
}

#[derive(Deserialize, Debug)]
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
    time: Res<Time>,
    ) {
    let mut player_transform = player_query.get_single_mut().unwrap();
    player_transform.translation += Vec3::X * 2.0 * time.delta_seconds();
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

    //let chunks = load_chunks();
    //for (chunk_pos, value) in chunks.iter() {
    //let chunk_corner_pos = chunk_pos.as_vec3() * CHUNK_SIZE;
    //let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
    //label: Some("instance data buffer"),
    //contents: bytemuck::cast_slice(value.as_slice()),
    //usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    //});

    //commands.spawn().insert_bundle((
    //meshes.get_handle(&ico_sphere),
    //Transform::from_xyz(0.0, 0.0, 0.0),
    //GlobalTransform::default(),
    //InstanceBuffer {
    //buffer,
    //length: value.len(),
    //},
    //ChunkPos(chunk_pos.clone()),
    //Visibility{ is_visible: true },
    //ComputedVisibility::default(),
    //Aabb::from_min_max(chunk_corner_pos, chunk_corner_pos + CHUNK_SIZE)
    //));
    //}
    let view_radius = ViewRadiusResource{ radius: 1.0 };
    commands.insert_resource(view_radius);
    let mut catalog = Catalog::new(
        "catalog_gaia_dr3_small".to_string(),
        ico_sphere,
        &mut commands,
    );
    catalog.particle_loader.update_chunks(&mut commands, render_device, Vec3::ZERO, view_radius.radius);
    commands.insert_resource(catalog);
    //
    // let octree_path = "/home/steam/git/stellar_coordinates_test/data/catalogs/catalog_gaia_dr3_small/catalog/gaia-dr3-small";
    // let metadata_file_path: PathBuf = [octree_path, METADATA_FILE].iter().collect();
    //let chunk_loader = ChunkLoader::new(
    //metadata_file_path,
    //meshes.get_handle(&ico_sphere),
    //Vec3::ZERO
    //);

    //commands.insert_resource(
    //chunk_loader
    //);

    let controller = FpsCameraController {
        smoothing_weight: 0.6,
        translate_sensitivity: 0.1,
        mouse_rotate_sensitivity: Vec2::splat(0.001),
        ..Default::default()
    };
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert_bundle(FpsCameraBundle::new(
            controller,
            PerspectiveCameraBundle::default(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1., 0., 0.),
        ));

    commands.spawn()
        .insert(Player())
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: view_radius.radius, subdivisions: 5  })),
            ..Default::default()
        })
        .insert_bundle(TransformBundle::identity());

    //let mesh_close = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 5 }));
    //let mesh_far = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 0 }));

    //LOD_map.0.push((0, mesh_close));
    //LOD_map.0.push((1, mesh_far));
}
