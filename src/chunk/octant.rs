use byteorder::{ReadBytesExt, BigEndian};
use std::{
    io::{self, Read},
};
use bevy::render::primitives::Aabb;

use crate::chunk::util::GITP;

#[derive(Debug)]
struct MetadataFile {
    #[allow(dead_code)]
    token: i32,
    #[allow(dead_code)]
    version: i32,
    size: i32,
}

impl MetadataFile {
    fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        //let mut cursor = Cursor::new(value);
        let token = reader.read_i32::<BigEndian>()?;
        let version;
        let size;
        if token < 0 {
            version = reader.read_i32::<BigEndian>()?;
            size = reader.read_i32::<BigEndian>()?;
        } else {
            version = 0;
            size = token;
        }

        Ok(MetadataFile {
            token,
            version,
            size,
        })
    }
}


pub const OCTANT_CHILDREN_COUNT: usize = 8;
#[derive(Debug)]
pub struct Octant {
    pub octant_id: usize,
    pub aabb: Aabb,
    pub children: [i64; OCTANT_CHILDREN_COUNT],
    pub depth: i32,
    pub cumulative_star_count: i32,
    pub star_count: i32,
    pub child_count: i32,
}

impl Octant {
    pub fn iter_from_reader(mut reader: &mut impl Read) -> impl Iterator<Item=Self> + '_ {
        let particle_file = MetadataFile::from_reader(&mut reader).unwrap(); 
        return (0..particle_file.size)
            .filter_map(move |_| Self::from_reader(&mut reader).ok())
    }

    fn from_reader(reader: &mut impl Read) -> io::Result<Self> {
        let octant_id = reader.read_i64::<BigEndian>()?;
        let x = reader.read_f32::<BigEndian>()? * (GITP as f32);
        let y = reader.read_f32::<BigEndian>()? * (GITP as f32);
        let z = reader.read_f32::<BigEndian>()? * (GITP as f32);

        let dx = (reader.read_f32::<BigEndian>()? / 2.0) * (GITP as f32);
        let dy = (reader.read_f32::<BigEndian>()? / 2.0)  * (GITP as f32);
        let dz = (reader.read_f32::<BigEndian>()?) / 2.0 * (GITP as f32);

        let mut children = [0; OCTANT_CHILDREN_COUNT];
        for i in 0..OCTANT_CHILDREN_COUNT {
            children[i] = reader.read_i64::<BigEndian>()?;
        }

        let depth = reader.read_i32::<BigEndian>()?;
        let cumulative_star_count = reader.read_i32::<BigEndian>()?;
        let star_count = reader.read_i32::<BigEndian>()?;
        let child_count = reader.read_i32::<BigEndian>()?;


        return Ok(Octant {
            octant_id: octant_id.try_into().unwrap(),
            aabb: Aabb::from_min_max(
                [x - dx, y - dy, z - dz].into(),
                [x + dx, y + dy, z + dz].into()
            ),
            children,
            depth,
            cumulative_star_count,
            star_count,
            child_count
        });

    }
}
