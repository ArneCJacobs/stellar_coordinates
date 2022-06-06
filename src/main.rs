use rayon::prelude::*;
use smooth_bevy_cameras::{controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin}, LookTransformPlugin};
use serde::Deserialize;
use std::fs::File;
use flate2::read::GzDecoder;
use bevy::prelude::*;

use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_debug_lines::*;
use bevy::render::primitives::Aabb;
use GPUInstanceing::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};
use itertools::Itertools;
use std::collections::HashMap;
use bevy::math::{BVec3A, Vec3A};
#[macro_use]
extern crate lazy_static;

mod cursor;
mod GPUInstanceing;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin::new()) // in game inspector
        .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        //.register_inspectable::<InstanceData>() // allows InstanceData to be inspected in egui
        //.register_inspectable::<InstanceMaterialData>() // allows InstanceData to be inspected in egui
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

fn to_bvec3(bitmask: u8) -> BVec3 {
    BVec3::new(
        (bitmask & 0b100) != 0,
        (bitmask & 0b010) != 0,
        (bitmask & 0b001) != 0,
    )
}

fn draw_bounding_box(lines: &mut ResMut<DebugLines>, aabb: &Aabb) {
    let min = aabb.min().into();
    let max = aabb.max().into();

    let connections = [
        (0b000, 0b100),
        (0b000, 0b010),
        (0b000, 0b001),

        (0b100, 0b110),
        (0b100, 0b101),

        (0b010, 0b110),
        (0b010, 0b011),

        (0b001, 0b101),
        (0b001, 0b011),

        (0b011, 0b111),
        (0b101, 0b111),
        (0b110, 0b111),
    ];

    for (from, to) in connections {
        lines.line_colored(
            Vec3::select(to_bvec3(from), min, max),
            Vec3::select(to_bvec3(to), min, max),
            0.0,
            Color::GREEN
        );
    }
}

const CHUNK_SIZE: f32 = 50.0;
const LIMIT: u32 = 3_000_000;
const SCALE: f32 = 1.0;
const STAR_COUNT: u32 = 3_000_000;

lazy_static!{
    static ref CHUNKS: HashMap<IVec3, Vec<InstanceData>> = {
        let file = File::open("./data/stars_transformed.csv.gz").expect("Could not open file");
        let decoder = GzDecoder::new(file);
        let reader = csv::ReaderBuilder::new().from_reader(decoder);

        let mut index = 0;
        let mut stars: Vec<InstanceData> = vec![];
        for record in reader.into_deserialize::<Pos>() {
            if let Ok(star_pos) = record {
                print!("\r                                            ");
                print!("\r {:06}/{} {:.3}%",index, STAR_COUNT, (index as f32) / (STAR_COUNT as f32) * 100f32);

                let star_pos_inst = InstanceData {
                    position: Vec3::new(star_pos.x * SCALE, star_pos.z * SCALE, star_pos.y * SCALE),
                    scale: 1.0,
                    color:  Color::hex("ffd891").unwrap().as_rgba_f32(),
                };
                stars.push(star_pos_inst);

            }
            //let star_pos: Pos = record.unwrap();
            index += 1;
            if index >= LIMIT {
                break;
            }
        }
        return stars.into_iter().into_group_map_by(|star_pos| {
            (star_pos.position / CHUNK_SIZE).floor().as_ivec3()
        });
    };
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


    //let min = stars.par_iter()
        //.map(|instance_data| instance_data.position)
        //.reduce(|| Vec3::ZERO, |accum, item| accum.min(item));

    //let max = stars.par_iter()
        //.map(|instance_data| instance_data.position)
        //.reduce(|| Vec3::ZERO, |accum, item| accum.max(item));
    let ico_sphere = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 0 }));

    for (key, value) in CHUNKS.iter() {
        let key = key.as_vec3() * CHUNK_SIZE;
        commands.spawn().insert_bundle((
                meshes.get_handle(&ico_sphere),
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                InstanceMaterialData(
                    &value
                ),
                Visibility{ is_visible: true },
                ComputedVisibility::default(),
                Aabb::from_min_max(key, key + CHUNK_SIZE)
        ));

    }


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
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1., 0., 0.),
        ));

}
