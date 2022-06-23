use std::io::BufReader;
use std::{
    fs::File,
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use bevy::math::Vec3;
use bevy::ecs::bundle::Bundle;
use bevy::render::primitives::Aabb;
use bevy::prelude::*;

pub mod octant;
pub mod particle;
pub mod util;


use crate::chunk::octant::Octant;
use crate::chunk::octant::OCTANT_CHILDREN_COUNT;
use crate::chunk::particle::Particle;
use crate::chunk::util::METADATA_FILE;
use crate::gpu_instancing::InstanceBuffer;

struct OctTree {
    octants: Vec<Octant>,
}

impl OctTree {
    pub fn new(mut octants: Vec<Octant>) -> Self {
        // sets the children field to be the index of the child rather then the id
        let mut octant_id_to_index: HashMap<i64, usize> = HashMap::new();
        for (index, octant) in octants.iter().enumerate() {
            octant_id_to_index.insert(octant.octant_id, index);
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

        OctTree {
            octants: octants
        }
    }

    pub fn from_file<P: AsRef<Path>>(metadata_path: P) -> Self {
        let metadata_file = File::open(metadata_path).expect("could not open metadata file");
        let mut reader = BufReader::new(metadata_file);
        let octants: Vec<Octant> = Octant::iter_from_reader(&mut reader).collect();
        return Self::new(octants);
    }

    //pub fn load_octants(&self, pos: Vec3, view_radius: f32) -> impl Iterator<Item=&Octant> {
        //for octant in self.octants.iter() {
            //println!("{:?}", octant);
        //}
        //todo!();
    //}

    //fn load_chunks(&self, pos: Vec3, render_distance: f32, max_stars: u32) -> 

}

#[derive(Bundle)]
pub struct Chunk {
    pub aabb: Aabb,
    pub instance_data: InstanceBuffer,
    #[bundle]
    pub transform_bundle: TransformBundle,
    pub mesh: Handle<Mesh>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Chunk {
    pub fn new(min: Vec3, max: Vec3, mesh: Handle<Mesh>, instance_data: InstanceBuffer) -> Self {
        Chunk {
            aabb: Aabb::from_min_max(min, max),
            instance_data,
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

pub struct ChunkLoader {
    octree: OctTree,
    mesh: Handle<Mesh>,
    last_pos: Vec3,
}

impl ChunkLoader {
    pub fn new<P: AsRef<Path>>(octee_path: P, mesh: Handle<Mesh>, start_pos: Vec3) -> Self {
        ChunkLoader {
            mesh,
            last_pos: start_pos,
            octree: OctTree::from_file(octee_path),
        }
    }


    pub fn print(&self) {
        let temp = vec![5, 12, 8, 16, 10, 11, 7, 13, 9];
        for (index, octant) in self.octree.octants.iter().enumerate().take(20).filter(|(i, _)| temp.contains(&i)) {
            println!("{:?}, {:?}", index, octant);
        }
    }

    //fn load_chunks(view_radius: f32) -> impl Iterator<Item=Chunk> {
        //todo!();
    //}
}

pub struct Catalog {
    //TODO should be given base path, and initiate Octree, ChunkLoader and load files contained in
    //octant when needed.
}
