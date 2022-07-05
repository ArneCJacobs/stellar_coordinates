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
use serde::Deserialize;
use serde_json;
use bit_set::BitSet;

pub mod octant;
pub mod particle;
pub mod util;


use crate::chunk::octant::Octant;
use crate::chunk::octant::OCTANT_CHILDREN_COUNT;
use crate::chunk::particle::Particle;
use crate::gpu_instancing::InstanceBuffer;
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
    pub instance_data: InstanceBuffer,
    #[bundle]
    pub transform_bundle: TransformBundle,
    pub mesh: Handle<Mesh>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Chunk {
    pub fn new(octant_id: usize, min: Vec3, max: Vec3, mesh: Handle<Mesh>, instance_data: InstanceBuffer) -> Self {
        Chunk {
            octant_id: OctantId(octant_id),
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
    fn load_octants(&mut self, sphere: Sphere) -> (Vec<&Octant>, Vec<&Octant>) {
        let mut new_octants = vec![];
        let mut unloaded_octants = vec![];
        self.new_octants.clear();

        for (index, octant) in self.octree.search_octants(sphere) {
            self.new_octants.insert(index);
            if !self.loaded_octants.contains(index) {
                new_octants.push(octant);
            } 
        }

        self.loaded_octants.difference_with(&self.new_octants);

        for index_unloaded in self.loaded_octants.iter() {
            unloaded_octants.push(&self.octree.octants[index_unloaded]);
        }

        std::mem::swap(&mut self.new_octants, &mut self.loaded_octants);

        return (new_octants, unloaded_octants);
        
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
    particles_dir_path: PathBuf,
    buffered_octant_loader: BufferedOctantLoader,
}


const CATALOGS_DIR: &'static str = "./data/catalogs/";

impl Catalog {
    pub fn new(name: String) -> Self {
        let catalog_dir: PathBuf = [CATALOGS_DIR, name.as_str()].iter().collect();

        let name = name.replace("_", "-"); // who though this was a good naming convention
        let mut catalog_description = catalog_dir.join(name);
        catalog_description.set_extension("json");

        let catalog_description_file = File::open(catalog_description).unwrap();
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

        Catalog {
            particles_dir_path,
            buffered_octant_loader: BufferedOctantLoader::new(octree),
        }
    }    

}

#[derive(Deserialize, Debug)]
struct CatalogData {
    files: Vec<PathBuf>
}


struct ParticleLoader {
    buffered_octant_loader: BufferedOctantLoader,


}

impl ParticleLoader {
    fn manage_chunks(&mut self, mut commands: &mut Commands, chunks: Query<(Entity, &OctantId)>, pos: Vec3, radius: f32) {
        let sphere = Sphere {
            center: pos.into(),
            radius
        };

        let (new_octants, unloaded_octants) = self.buffered_octant_loader.load_octants(sphere);
        for octant in unloaded_octants {
             
        }

    }
}
