use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::thread::JoinHandle;
use bevy::math::Vec3;
use bevy::ecs::bundle::Bundle;
use bevy::render::primitives::Aabb;
use bevy::prelude::*;
use bevy::render::render_resource::{BufferInitDescriptor, BufferUsages};
use bevy::render::renderer::RenderDevice;
use serde::Deserialize;
use serde_json;
use bit_set::BitSet;
use vec_map::VecMap;
use crossbeam_channel::{unbounded, Receiver, Sender};

pub mod octant;
pub mod particle;
pub mod util;


use crate::chunk::octant::Octant;
use crate::chunk::octant::OCTANT_CHILDREN_COUNT;
use crate::chunk::particle::Particle;
use crate::gpu_instancing::{InstanceBuffer, InstanceData};
use bevy::render::primitives::Sphere;

pub struct OcTree {
    octants: Vec<Octant>,
    root_index: usize,
}

impl OcTree {
    pub fn new(mut octants: Vec<Octant>) -> Self {
        // sets the children field to be the index of the child rather then the id
        let mut octant_id_to_index: HashMap<i64, usize> = HashMap::new();
        let mut root_index_opt = None;
        for (index, octant) in octants.iter().enumerate() {
            octant_id_to_index.insert(octant.octant_id.try_into().unwrap(), index);
            if octant.depth == 0 {
                // TODO to be completely accurate, it should be checked if root_index_opt is None
                // when assigning a new value and panic if not
                root_index_opt = Some(index);
            }
        }

        for octant in octants.iter_mut() {
            for i in 0..OCTANT_CHILDREN_COUNT {
                if octant.children[i] == -1 {
                    continue;
                }
                let temp = octant_id_to_index.get(&octant.children[i]).unwrap();
                octant.children[i] = (*temp) as i64;
            }
        }

        OcTree {
            octants,
            root_index: root_index_opt.expect("No root element was found")
        }
    }

    pub fn from_file<P: AsRef<Path>>(metadata_path: P) -> Self {
        let metadata_file = File::open(metadata_path).expect("could not open metadata file");
        let mut reader = BufReader::new(metadata_file);
        let octants: Vec<Octant> = Octant::iter_from_reader(&mut reader).collect();
        return Self::new(octants);
    }

    fn get_root(&self) -> &Octant {
        &self.octants[self.root_index]
    }

    fn search_octants(&self, sphere: Sphere) -> Vec<(usize, &Octant)> { // return iterator instead
        let mut stack = vec![(self.root_index ,self.get_root())];
        let mut intersected = vec![];
        
        while let Some((index, octant)) = stack.pop() {
            if sphere.intersects_obb(&octant.aabb, &Mat4::IDENTITY) {
                intersected.push((index, octant));
                for &child_index in octant.children.iter() {
                    if child_index != -1 {
                        stack.push((child_index as usize, &self.octants[child_index as usize]));
                    }
                }
            }
        }

        return intersected;
    }

}

#[derive(Component)]
pub struct OctantId(usize);

#[derive(Bundle)]
pub struct Chunk {
    pub octant_id: OctantId,
    pub aabb: Aabb,
    pub instance_buffer: InstanceBuffer,
    #[bundle]
    pub transform_bundle: TransformBundle,
    pub mesh: Handle<Mesh>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Chunk {
    pub fn new(octant_id: usize, aabb: Aabb, mesh: Handle<Mesh>, instance_buffer: InstanceBuffer) -> Self {
        Chunk {
            octant_id: OctantId(octant_id),
            aabb,
            instance_buffer,
            transform_bundle: TransformBundle::identity(),
            mesh,
            visibility: Visibility { is_visible: true },
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

fn read_particle_file() {
    let file = File::open("./particles_000000.bin").expect("Could not open file");
    let mut reader = BufReader::new(file);
    //let particle_file = ParticleFile::from_reader(reader);
    //let particle_file: ParticleFile = options.deserialize_from(reader).unwrap();
    //println!("{:?}", particle_file);
    for particle in Particle::iter_from_reader(&mut reader).take(1) {
        println!("{:?}", particle);
    }
}

pub struct BufferedOctantLoader {
    octree: OcTree,
    loaded_octants: BitSet,
    new_octants: BitSet,
}

impl BufferedOctantLoader {
    fn new(octree: OcTree) -> Self {
        BufferedOctantLoader {
            octree,
            loaded_octants: BitSet::new(),
            new_octants: BitSet::new(),
        }
    }

    // sphere contains the position and the view radius, and will be used for collision with the
    // aabb in the octree
    fn load_octants(&mut self, sphere: Sphere) -> (Vec<(usize,&Octant)>, Vec<(usize,&Octant)>) {
        let mut new_octants = vec![];
        let mut unloaded_octants = vec![];
        self.new_octants.clear();

        for (index, octant) in self.octree.search_octants(sphere) {
            self.new_octants.insert(index);
            if !self.loaded_octants.contains(index) {
                new_octants.push((index, octant));
            } 
        }

        self.loaded_octants.difference_with(&self.new_octants);

        for index_unloaded in self.loaded_octants.iter() {
            unloaded_octants.push((index_unloaded, &self.octree.octants[index_unloaded]));
        }

        std::mem::swap(&mut self.new_octants, &mut self.loaded_octants);

        return (new_octants, unloaded_octants);
        
    }
}

pub struct Catalog {
    pub particle_loader: ParticleLoader,
}


const CATALOGS_DIR: &'static str = "./data/catalogs/";

impl Catalog {
    pub fn new(
        name: String, 
        initial_mesh: Handle<Mesh>,
    ) -> Self {
        let catalog_dir: PathBuf = [CATALOGS_DIR, name.as_str()].iter().collect();

        let name = name.replace("_", "-"); // who though this was a good naming convention
        let mut catalog_description = catalog_dir.join(name);
        catalog_description.set_extension("json");

        let catalog_description_file = File::open(catalog_description.clone()).expect(&format!("Could not find/open catalog, {}", catalog_description.to_str().unwrap()).to_string());
        let catalog_data: CatalogData = serde_json::from_reader(catalog_description_file).unwrap();

        let metadata_path = catalog_data.files
            .iter()
            .filter(|path| path.file_name().is_some())
            .filter(|path| path.file_name().unwrap() == "metadata.bin")
            .next()
            .expect("No metadata file found");

        let metadata_path = catalog_dir.join(metadata_path);

        let particles_dir_path = catalog_data.files
            .iter()
            .filter(|path| path.file_name().is_some())
            .filter(|path| path.file_name().unwrap() == "particles")
            .next()
            .expect("No particles directory found");

        let particles_dir_path = catalog_dir.join(particles_dir_path);

        let octree = OcTree::from_file(metadata_path); 
        let particle_loader = ParticleLoader::new(
            BufferedOctantLoader::new(octree),
            initial_mesh, particles_dir_path
        );

        Catalog {
            particle_loader,
        }
    }    

}

#[derive(Deserialize, Debug)]
struct CatalogData {
    files: Vec<PathBuf>
}

pub struct OctantData {
    pub octant_id: usize,
    pub octant_index: usize,
    pub instance_data_opt: Option<Vec<InstanceData>>,
}

pub struct ParticleLoader {
    buffered_octant_loader: BufferedOctantLoader,
    loaded_octants: VecMap<Entity>,
    initial_mesh: Handle<Mesh>,
    // sends octant id to the loader thread
    loader_thread_sender: Sender<OctantData>,
    // receives particle data send from the loader thread
    main_tread_receiver: Receiver<OctantData>,
    loader_thread_join_handle: JoinHandle<()>,
}



#[derive(Deref)]
struct StreamReceiver(Receiver<OctantData>);

impl ParticleLoader {
    fn new(
        buffered_octant_loader: BufferedOctantLoader, 
        initial_mesh: Handle<Mesh>, 
        particles_dir_path: PathBuf
    ) -> Self {

        // sends and receives octant ids to and from the loader thread
        let (sender_to, receiver_to): (Sender<OctantData>, Receiver<OctantData>) = unbounded();
        // sends and receives the loaded data back to the main thread
        let (sender_from, receiver_from): (Sender<OctantData>, Receiver<OctantData>) = unbounded();

        let join_handle = std::thread::Builder::new().name("Loader thread".to_string()).spawn(move || {
            while let Ok(mut octant_data) = receiver_to.recv() {
                let mut particle_file_path = particles_dir_path.join(format!("particles_{:0>6}", octant_data.octant_id.to_string()));
                particle_file_path.set_extension("bin");
                let particle_file = File::open(particle_file_path.clone());
                if let Err(error) = particle_file {
                    eprintln!("An error occured while opening particle file: {:?}, error: {}", particle_file_path.to_str(), error);
                    break;
                }
                let instance_data: Vec<InstanceData> = Particle::iter_from_reader(&mut particle_file.unwrap())
                    .map(|particle: Particle| {
                        InstanceData {
                            position: Vec3::new(
                                particle.x as f32,
                                particle.y as f32,
                                particle.z as f32,
                            ),
                            scale: 1.0,
                            color:  Color::hex("ffd891").unwrap().as_rgba_f32(),
                        }
                    }).collect();
                octant_data.instance_data_opt = Some(instance_data);
                if let Err(_) = sender_from.try_send(octant_data) {
                    eprintln!("Loader thread: Unexpected stop, main thread no longer receiving");
                    break;
                }
            }

        }).unwrap();

        // commands.insert_resource(StreamReceiver(receiver_from)); //TODO make system which receives this data

        ParticleLoader {
            buffered_octant_loader,
            loaded_octants: VecMap::new(),
            loader_thread_sender: sender_to,
            initial_mesh,
            main_tread_receiver: receiver_from,
            loader_thread_join_handle: join_handle,
        }
    }

    // adds and removes chunks from bevy when the corresponding octant is out of render distance
    // in the octree
    pub fn update_chunks(
        &mut self, 
        commands: &mut Commands, 
        render_device: Res<RenderDevice>, 
        pos: Vec3, 
        radius: f32,
    ) {
        if self.loader_thread_join_handle.is_finished() {
            panic!("Loaded thread stopped running!");
        }
        let sphere = Sphere {
            center: pos.into(),
            radius

        };

        // received particle data from loader thread and add it to bevy
        for octant_data in self.main_tread_receiver.try_iter() {
            let octant_opt = self.buffered_octant_loader.octree.octants.get(octant_data.octant_index);
            if let Some(instance_data) = octant_data.instance_data_opt {
                let instance_buffer = InstanceBuffer {
                    buffer: render_device.create_buffer_with_data(&BufferInitDescriptor{
                        label: Some(format!("Octant {}", octant_data.octant_id).as_str()),
                        contents: bytemuck::cast_slice(instance_data.as_slice()),
                        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    }),
                    length: instance_data.len(),
                };
                let chunk = Chunk::new(
                    octant_data.octant_id,
                    octant_opt.unwrap().aabb.clone(),
                    self.initial_mesh.clone(),
                    instance_buffer,
                );
                let entity_command = commands.spawn_bundle(chunk);
                let entity = entity_command.id();
                self.loaded_octants.insert(octant_data.octant_index, entity);
            }
        }

        // get which octant need to be loaded or unloaded
        let (new_octants, unloaded_octants) = self.buffered_octant_loader.load_octants(sphere);
        // send octant that need to be loaded to loader thread
        for (index, octant) in new_octants {
            let octant_data = OctantData {
                octant_id: octant.octant_id,
                octant_index: index,
                instance_data_opt : None,
            };
            if let Err(error) = self.loader_thread_sender.try_send(octant_data) {
                panic!("Could not send octant load request to worker thread, reason: {}", error);
            }
        }

        // octants that need to be unloaded are removed from entities
        for (index, _octant) in unloaded_octants {
            let values = self.loaded_octants.values().collect::<Vec<&Entity>>();
            let error_msg = format!("octant with id: {} was not found, loaded octants: {:?}", index, values);
            let entity = self.loaded_octants.remove(index).expect(error_msg.as_str());
            commands.entity(entity).despawn();
        }

    }
}
