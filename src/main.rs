use std::f32::consts::PI;

#[allow(unused_imports)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{primitives::{Aabb, Sphere}, renderer::RenderDevice}, window::PresentMode,
};

#[allow(unused_imports)]
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use chunk::{BufferedOctantLoader, util::{DATA_SCALE, PARSEC, LIGHT_YEAR, ASTRONOMICAL_UNIT}};
use itertools::Itertools;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use vec_map::VecMap;


use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::chunk::Catalog;

mod chunk;
mod cursor;
mod util;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Mailbox,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        // .add_plugin(WorldInspectorPlugin::new()) // in game inspector
        // .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
        ]))
        // .add_plugin(DebugLinesPlugin::default())
        // .add_system(draw_bounding_box_system)
        .add_system(catalog_system)
        .add_system(lod_system.after(catalog_system))
        .add_system(egui_system)
        .add_startup_system(setup)
        .run();
}

const RADIAN_TO_HOURS: f32 = PI / 12.0;
const RADIAN_TO_MINUTES: f32 = PI / 720.0;
const RADIAN_TO_SECONDS: f32 = PI / 43_200.0;

fn to_hour_minute_seconds(radians: f32) -> [f32; 3] {
    let hours = (radians / RADIAN_TO_HOURS).floor();
    let mut remaider = radians % RADIAN_TO_HOURS;
    let minutes = (remaider / RADIAN_TO_MINUTES).floor();
    remaider = remaider % RADIAN_TO_MINUTES;
    let seconds = remaider / RADIAN_TO_SECONDS;
    return [hours, minutes, seconds];
}
fn pretty_int(i: u64) -> String {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    return s;
}


fn egui_system(
    mut egui_context: ResMut<EguiContext>,
    player_query: Query<&Transform, With<Player>>,
    mut game_settings: ResMut<GameSettings>,
    star_count: Res<StarCount>,
    mut fps_camera_controller_query: Query<&mut FpsCameraController>, 
) {
    let mut controller = fps_camera_controller_query.get_single_mut().unwrap();
    let mut pos: Vec3 = player_query.get_single().unwrap().translation;
    let radius = pos.length();
    let declination = (pos.y / radius).asin().to_degrees();
    let mut right_ascension = (pos.x / radius).atan2(pos.z / radius);
    right_ascension = (right_ascension + 2.0 * PI) % (2.0 * PI);
    egui::Window::new("").show(egui_context.ctx_mut(), |ui| {
        ui.heading("Options");
        ui.add(egui::Slider::new(&mut game_settings.view_radius, 1.0..=1000.0).text("View raduis"));
        ui.add(egui::Slider::new(&mut controller.translate_sensitivity, 0.01..=100.0).text("Speed"));
        ui.heading("Information");
        pos /= DATA_SCALE as f32; // convert x, y, z to parsecs 
        ui.label(format!("x: {:.2}, y: {:.2}, z: {:.2}", pos.x , pos.y, pos.z));
        ui.label(format!("δ (DEC): {:.3}°", declination));

        let [hours, minutes, seconds] = to_hour_minute_seconds(right_ascension);
        ui.label(format!("α (RA): {:.0}h, {:.0}m, {:.2}s", hours, minutes, seconds));

        let parsecs = (radius as f64) / DATA_SCALE;
        let light_years = parsecs * (PARSEC / LIGHT_YEAR);
        let astronomical_units = parsecs * (PARSEC / ASTRONOMICAL_UNIT);
        ui.label(format!("Distance: {:.2e} ly, {:.2e} pc, {:.2e} AU,", light_years, parsecs, astronomical_units));

        ui.label(format!("Loaded star count: {}", pretty_int(star_count.0)));
    });
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
    player_query: Query<&Transform, With<Player>>,
    mut catalog: ResMut<Catalog>,
    view_radius: Res<GameSettings>,
    mut octant_map: ResMut<OctantMap>,
    mut star_count: ResMut<StarCount>,
    ) {
    let player_transform = player_query.get_single().unwrap();
    catalog.particle_loader.update_chunks(
        &mut commands, 
        render_device, 
        player_transform.translation, 
        view_radius.view_radius,
        &mut octant_map,
        &mut star_count.0,
    );
     
}

struct LODdata{
    buffered_octant_loader: BufferedOctantLoader,
    lod_radius: f32,
    mesh_close: Handle<Mesh>,
    mesh_far: Handle<Mesh>,
}

// maps an octant index to it's corresponding entity
pub struct OctantMap(VecMap<Entity>);

fn lod_system(
    mut commands: Commands,
    mut lod_data: ResMut<LODdata>,
    player_query: Query<&Transform, With<Player>>,
    octant_map: Res<OctantMap>,
) {
    let player_transform = player_query.get_single().expect("No player found");
    let lod_sphere = Sphere {
        radius: lod_data.lod_radius,
        center: player_transform.translation.into() 
    };

    let mesh_close = lod_data.mesh_close.clone();
    let mesh_far = lod_data.mesh_far.clone();

    let (close_octants, far_octants) = lod_data.buffered_octant_loader.load_octants(lod_sphere); 
    for (index, _) in close_octants {
        let entity_id = octant_map.0.get(index).unwrap();
        commands.entity(*entity_id)
            .insert(mesh_close.clone());
    }

    for (index, _) in far_octants {
        let entity_id = octant_map.0.get(index).unwrap();
        commands.entity(*entity_id)
            .insert(mesh_far.clone());
    }
}

#[derive(Clone, Copy)]
struct GameSettings {
    view_radius: f32,
    camera_speed: f32,
}

struct StarCount(u64);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    render_device: Res<RenderDevice>,
) {
    let mut octant_map = OctantMap(VecMap::new());
    let radius = 0.1f32;
    let ico_sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius,
        subdivisions: 0,
    }));

    let ico_sphere_close = meshes.add(Mesh::from(shape::Icosphere {
        radius,
        subdivisions: 0,
    }));

    let args = std::env::args().collect_vec();
    let catalog_name = args.get(1).expect("Catalog name cannot be empty, please provide it as the first argument");
    let game_settings = GameSettings{ 
        view_radius: 50.0, 
        camera_speed: 0.1 
    };
    commands.insert_resource(game_settings);
    let mut catalog = Catalog::new(
        catalog_name.to_string(),
        ico_sphere.clone(),
    );
    let mut star_count = 0;
    catalog.particle_loader.update_chunks(&mut commands, render_device, Vec3::ZERO, game_settings.view_radius, &mut octant_map, &mut star_count);

    let lod_buffered_octant_loader = BufferedOctantLoader::new(
        catalog.particle_loader.buffered_octant_loader.octree.clone()
    );

    let lod_data: LODdata = LODdata { 
        buffered_octant_loader: lod_buffered_octant_loader, 
        lod_radius: 1.0, 
        mesh_close: ico_sphere_close, 
        mesh_far: ico_sphere.clone() 
    };

    commands.insert_resource(catalog);
    commands.insert_resource(lod_data);
    commands.insert_resource(octant_map);
    commands.insert_resource(StarCount(star_count));



    // camera
    let controller = FpsCameraController {
        smoothing_weight: 0.6,
        translate_sensitivity: game_settings.camera_speed,
        mouse_rotate_sensitivity: Vec2::splat(0.001),
        ..Default::default()
    };
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert(Player()) // mark the camera as the main player
        .insert_bundle(FpsCameraBundle::new(
            controller,
            PerspectiveCameraBundle::default(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.000, 0.001, 1.),
        ));
}
