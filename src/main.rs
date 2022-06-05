use rayon::prelude::*;
use smooth_bevy_cameras::{controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin}, LookTransformPlugin};
use bytemuck::{Pod, Zeroable};
use serde::Deserialize;
use std::fs::File;
use flate2::read::GzDecoder;
use bevy::prelude::*;

use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_debug_lines::*;
use bevy::render::primitives::Aabb;
use GPUInstanceing::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};

mod cursor;
mod GPUInstanceing;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new()) // in game inspector
        .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        .register_inspectable::<InstanceData>() // allows InstanceData to be inspected in egui
        .register_inspectable::<InstanceMaterialData>() // allows InstanceData to be inspected in egui
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup)
        .add_system(draw_bounding_box_system)
        .run();
}

fn draw_bounding_box_system(
    mut lines: ResMut<DebugLines>,
    query: Query<&bevy::render::primitives::Aabb>
) {
    for aabb in query.iter() {
        draw_bounding_box(&mut lines, aabb);
    }

}

fn draw_bounding_box(lines: &mut ResMut<DebugLines>, aabb: &Aabb) {
    let (min_x, min_y, min_z): (f32, f32, f32) = aabb.min().into();
    let (max_x, max_y, max_z): (f32, f32, f32) = aabb.max().into();
    lines.line_colored(Vec3::new(min_x, min_y, min_z), Vec3::new(max_x, min_y, min_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(min_x, min_y, min_z), Vec3::new(min_x, max_y, min_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(min_x, min_y, min_z), Vec3::new(min_x, min_y, max_z), 0.0, Color::GREEN);

    lines.line_colored(Vec3::new(max_x, min_y, min_z), Vec3::new(max_x, max_y, min_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(max_x, min_y, min_z), Vec3::new(max_x, min_y, max_z), 0.0, Color::GREEN);

    lines.line_colored(Vec3::new(min_x, max_y, min_z), Vec3::new(max_x, max_y, min_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(min_x, max_y, min_z), Vec3::new(min_x, max_y, max_z), 0.0, Color::GREEN);

    lines.line_colored(Vec3::new(min_x, min_y, max_z), Vec3::new(max_x, min_y, max_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(min_x, min_y, max_z), Vec3::new(min_x, max_y, max_z), 0.0, Color::GREEN);

    lines.line_colored(Vec3::new(max_x, max_y, min_z), Vec3::new(max_x, max_y, max_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(max_x, min_y, max_z), Vec3::new(max_x, max_y, max_z), 0.0, Color::GREEN);
    lines.line_colored(Vec3::new(min_x, max_y, max_z), Vec3::new(max_x, max_y, max_z), 0.0, Color::GREEN);
}

#[derive(Deserialize, Debug)]
struct Pos {
    x: f32,
    y: f32,
    z: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {

    let file = File::open("./data/stars_transformed.csv.gz").expect("Could not open file");
    //let file = File::open("./data/stars_big_transformed.csv.gz").expect("Could not open file");
    let decoder = GzDecoder::new(file);

    // load in stars
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(decoder);

    let mut index = 0;
    let star_count = 3_000_000;
    let scale = 1.0f32;
    let limit = 500_000;
    let mut stars: Vec<InstanceData> = vec![];
    for record in reader.into_deserialize::<Pos>() {
        if let Ok(star_pos) = record {
            print!("\r                                            ");
            print!("\r {:06}/{} {:.3}%",index, star_count, (index as f32) / (star_count as f32) * 100f32);

            let star_pos_inst = InstanceData {
                position: Vec3::new(star_pos.x * scale, star_pos.z * scale, star_pos.y * scale),
                scale: 1.0,
                color:  Color::hex("ffd891").unwrap().as_rgba_f32(),
            };
            stars.push(star_pos_inst);

        }
        //let star_pos: Pos = record.unwrap();
        index += 1;
        if index >= limit { 
            break;
        }
    }

    let min = stars.par_iter()
        .map(|instance_data| instance_data.position)
        .reduce(|| Vec3::ZERO, |accum, item| accum.min(item));

    let max = stars.par_iter()
        .map(|instance_data| instance_data.position)
        .reduce(|| Vec3::ZERO, |accum, item| accum.max(item));

    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 0 })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        InstanceMaterialData(
            stars
        ),
        Visibility::default(),
        ComputedVisibility::default(),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        //NoFrustumCulling,
        Aabb::from_min_max(min, max)

    ));

    // camera
    //commands.spawn_bundle(PerspectiveCameraBundle {
        //transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        //..default()
    //});
    let controller = FpsCameraController {
        smoothing_weight : 0.6,
        translate_sensitivity: 0.1,
        mouse_rotate_sensitivity: Vec2::splat(0.001),
        ..Default::default()
    };
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert_bundle(FpsCameraBundle::new(
                controller,
                PerspectiveCameraBundle::default(),
                Vec3::new(-2.0, 5.0, 5.0),
                Vec3::new(0., 0., 0.),
        ));

}
