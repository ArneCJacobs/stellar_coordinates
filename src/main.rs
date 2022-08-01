use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{primitives::{Aabb, Sphere}, renderer::RenderDevice}, window::PresentMode,
};

#[allow(unused_imports)]
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use chunk::BufferedOctantLoader;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use vec_map::VecMap;

use crate::chunk::Catalog;
use crate::gpu_instancing::CustomMaterialPlugin;

mod chunk;
mod cursor;
mod gpu_instancing;
mod util;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Mailbox,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // .add_plugin(WorldInspectorPlugin::new()) // in game inspector
        .add_plugin(CustomMaterialPlugin) // for GPU instancing
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_system(cursor::cursor_grab_system)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            // FrameTimeDiagnosticsPlugin::FPS,
        ]))
        // .add_plugin(DebugLinesPlugin::default())
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            present_mode: PresentMode::Immediate,
            ..default()
        })
        // .add_system(draw_bounding_box_system)
        .add_system(catalog_system)
        .add_system(lod_system.after(catalog_system))
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
    player_query: Query<&Transform, With<Player>>,
    mut catalog: ResMut<Catalog>,
    view_radius: Res<ViewRadiusResource>,
    mut octant_map: ResMut<OctantMap>,
    ) {
    let player_transform = player_query.get_single().unwrap();
    catalog.particle_loader.update_chunks(
        &mut commands, 
        render_device, 
        player_transform.translation, 
        view_radius.radius,
        &mut octant_map
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
struct ViewRadiusResource {
    radius: f32,
}

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
        subdivisions: 1,
    }));

    let view_radius = ViewRadiusResource{ radius: 200.0 };
    commands.insert_resource(view_radius);
    let mut catalog = Catalog::new(
        "catalog_gaia_dr3_extralarge".to_string(),
        ico_sphere.clone(),
    );
    catalog.particle_loader.update_chunks(&mut commands, render_device, Vec3::ZERO, view_radius.radius, &mut octant_map);

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



    // camera
    let controller = FpsCameraController {
        smoothing_weight: 0.6,
        translate_sensitivity: 0.1,
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
            Vec3::new(1., 0., 1.),
        ));
}
