#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs::File;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        primitives::Aabb,
        render_resource::*,
        renderer::RenderDevice
    },
    window::PresentMode
};

use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
use bevy_prototype_debug_lines::*;
use flate2::read::GzDecoder;
use itertools::Itertools;
use serde::Deserialize;
use smooth_bevy_cameras::{controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin}, LookTransformPlugin};

use gpu_instancing::{CustomMaterialPlugin, InstanceData, InstanceMaterialData, InstanceMaterialDataBuffer};

mod cursor;
mod gpu_instancing;
mod util;

const CHUNK_SIZE: f32 = 50.0;
const LIMIT: u32 = 3_000_000;
const SCALE: f32 = 1.0;
const STAR_COUNT: u32 = 3_000_000;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin::new()) // in game inspector
        .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![FrameTimeDiagnosticsPlugin::FPS]))
        .add_plugin(DebugLinesPlugin::default())
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            // present_mode: PresentMode::Immediate,
            ..default()
        })

        .add_startup_system(setup)
        //.add_system(draw_bounding_box_system)
        .run();
}

fn draw_bounding_box_system(
    mut lines: ResMut<DebugLines>,
    query: Query<&bevy::render::primitives::Aabb>
) {
    for aabb in query.iter() {
        util::draw_bounding_box(&mut lines, aabb);
    }

}


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
    render_device: Res<RenderDevice>,
) {
    
    let ico_sphere = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 0 }));

    for (key, value) in CHUNKS.iter() {
        let key = key.as_vec3() * CHUNK_SIZE;
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(value.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        commands.spawn().insert_bundle((
                meshes.get_handle(&ico_sphere),
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                InstanceMaterialData(&value),
                InstanceMaterialDataBuffer(buffer),
                Visibility{ is_visible: true },
                ComputedVisibility::default(),
                Aabb::from_min_max(key, key + CHUNK_SIZE)
        ));

    }

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
