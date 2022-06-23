use std::io::BufReader;
use std::{
    fs::File,
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use bevy::math::Vec3;

pub mod octant;
pub mod particle;
pub mod util;


use crate::chunk::octant::Octant;
use crate::chunk::octant::OCTANT_CHILDREN_COUNT;
use crate::chunk::particle::Particle;
use crate::chunk::util::METADATA_FILE;

struct OctTree {
    octants: Vec<Octant>,
}

impl OctTree {
    fn new(mut octants: Vec<Octant>) -> Self {
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

    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let metadata_file = File::open(path).expect("could not open metadata file");
        let mut reader = BufReader::new(metadata_file);
        let octants: Vec<Octant> = Octant::iter_from_reader(&mut reader).collect();
        return Self::new(octants);
    }

    //fn load_chunks(&self, pos: Vec3, render_distance: f32, max_stars: u32) -> 

}

struct Chunk {
    pos: Vec3,
    d_pos: Vec3,
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


fn main() {
    let base_path = "./catalog_gaia_dr3_small/catalog/gaia-dr3-small/";
    //let base_path = "/home/steam/mount/secondary/gaiasky_datasets/catalog-gaia-dr3-extralarge/catalog/gaia-dr3-extralarge";
    let metadata_file_path: PathBuf = [base_path, METADATA_FILE].iter().collect();

    let oct_tree = OctTree::from_file(metadata_file_path);

    for octant in oct_tree.octants {
        println!("{:?}", octant);
    }

    //let file = File::open("./catalog-gaia-dr3-small.tar.gz").unwrap();
    //let decoder = GzDecoder::new(file);

    //let mut archive = Archive::new(decoder);
    //for file in archive.entries().unwrap() {
        //if let Ok(file) = file {
            //println!("{:?}", file.header());
        //} else if let Err(err) = file {
            //println!("{:?}", err);
        //}
    //}

}
