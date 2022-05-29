use bevy::prelude::*;
use serde::Deserialize;
use bevy_inspector_egui::WorldInspectorPlugin;
use crossbeam_channel::{bounded, Receiver};

//#[derive(Component)]
//struct Person;

//#[derive(Component)]
//struct Name(String);

//fn add_people(mut commands: Commands) {
    //commands.spawn().insert(Person).insert(Name("Eve Claessens".to_string()));
    //commands.spawn().insert(Person).insert(Name("Bart Coppens".to_string()));
    //commands.spawn().insert(Person).insert(Name("Deni Bitch".to_string()));
//}

//struct GreetTimer(Timer);

//fn greet_people(
    //time: Res<Time>, 
    //mut timer: ResMut<GreetTimer>,
    //query: Query<&Name, With<Person>>) {
    //if timer.0.tick(time.delta()).just_finished() {
        //for name in query.iter() {
            //println!("hello {}!", name.0)
        //}
    //}
//}

fn main() {
    App::new()
        .add_event::<StreamEvent>()
        .add_plugins(DefaultPlugins)
        //.add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        //.add_system(read_stream)
        //.add_system(spawn_star)
        //.add_plugin(HelloPlugin)
        .run();
}


//pub struct HelloPlugin;


//impl Plugin for HelloPlugin {
    //fn build(&self, app: &mut App) {
        //app.insert_resource(GreetTimer(Timer::from_seconds(2.0, true)))
            //.add_startup_system(add_people)
            //.add_system(greet_people);
    //}
//}
#[derive(Deref)]
struct StreamReceiver(Receiver<Vec<Pos>>);
struct StreamEvent(Vec<Pos>);

#[derive(Deserialize, Debug)]
struct Pos {
    x: f32,
    y: f32,
    z: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::default(), Vec3::Y),
        orthographic_projection: OrthographicProjection {
            scale: 0.01,
            ..default()
        },
        ..OrthographicCameraBundle::new_3d()
    });


    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path("./data/stars_big_transformed.csv")
        .unwrap();

    let mut index = 0;
    let star_count = 220996;
    for record in reader.deserialize() {
        let star_pos: Pos = record.unwrap();

        print!("\r                                            ");
        print!("\r {:06}/{} {:.3}%",index, star_count, (index as f32) / (star_count as f32) * 100f32);
        index += 1;
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.05,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("ffd891").unwrap(),
                // vary key PBR parameters on a grid of spheres to show the effect
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(star_pos.x, star_pos.y, star_pos.z),
            ..default()
        });
    }

}

//fn spawn_star(
    //mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    //mut reader: EventReader<StreamEvent>,
//) {

    //for event in reader.iter() {
        //println!("loaded stars");
        //let vector: &Vec<Pos> = &event.0;
        ////let star_pos = &event.0;
    //}
//}
