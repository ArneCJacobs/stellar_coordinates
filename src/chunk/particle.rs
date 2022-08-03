use byteorder::{ReadBytesExt, BigEndian};
use std::io::{self, Read};
use crate::chunk::util::GITPS;

use super::util::gaiasky_to_cartesian;

// https://gaia.ari.uni-heidelberg.de/gaiasky/docs/master/Data-streaming.html#version-2

fn rgba_from_float(float: f32) -> [u8; 4] {
    let [mut a, b, g, r] = float.to_be_bytes();
    a = ((a as f32) * 255./254.).floor() as u8;
    return [r, g, b, a];
}


#[derive(Debug)]
struct ParticleFile {
    #[allow(dead_code)]
    token: i32,
    #[allow(dead_code)]
    version: i32,
    size: i32,
}

impl ParticleFile {
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

        Ok(ParticleFile {
            token,
            version,
            size,
        })
    }
}


#[derive(Debug)]
pub struct Particle {
    pub x: f64, pub y: f64, pub z: f64,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub mu_alpha: f32, pub mu_delta: f32, pub rad_vel: f32,
    pub app_mag: f32, pub abs_mag: f32, pub color: [u8; 4], pub size: f32,
    pub hip_number: i32,
    pub source_id: i64,
    pub name_len: i32,
    pub names: Vec<String>,
}

impl Particle {
    pub fn iter_from_reader(mut reader: &mut impl Read) -> impl Iterator<Item=Self> + '_ {
        let particle_file = ParticleFile::from_reader(&mut reader).unwrap(); 
        return (0..particle_file.size)
            .filter_map(move |_| Self::from_reader(&mut reader).ok())


    }

    fn from_reader(reader: &mut impl Read) -> io::Result<Self> {
        let x = reader.read_f64::<BigEndian>()? * GITPS;
        let y = reader.read_f64::<BigEndian>()? * GITPS;
        let z = reader.read_f64::<BigEndian>()? * GITPS;
        // [x, y, z] = gaiasky_to_cartesian::<f64, [f64; 3], [f64; 3]>([x, y, z]);

        let vx = reader.read_f32::<BigEndian>()? * (GITPS as f32);
        let vy = reader.read_f32::<BigEndian>()? * (GITPS as f32);
        let vz = reader.read_f32::<BigEndian>()? * (GITPS as f32);
        // [vx, vy, vz] = gaiasky_to_cartesian::<f32, [f32; 3], [f32; 3]>([vx, vy, vz]);

        let mu_alpha = reader.read_f32::<BigEndian>()?;
        let mu_delta = reader.read_f32::<BigEndian>()?;
        let rad_vel = reader.read_f32::<BigEndian>()?;

        let app_mag = reader.read_f32::<BigEndian>()?;
        let abs_mag = reader.read_f32::<BigEndian>()?;
        let color = rgba_from_float(reader.read_f32::<BigEndian>()?);
        let size = reader.read_f32::<BigEndian>()?;

        let hip_number = reader.read_i32::<BigEndian>()?;
        let source_id = reader.read_i64::<BigEndian>()?;
        let name_len = reader.read_i32::<BigEndian>()?;

        let mut name: Vec<u16> = Vec::with_capacity(name_len as usize);
        for _ in 0..name_len {
            name.push(reader.read_u16::<BigEndian>()?);
        }

        let names = String::from_utf16(name.as_slice()).unwrap()
            .split("|")
            .map(|s| String::from(s))
            .collect();


        return Ok(Particle {
            x, y, z,
            vx, vy, vz,
            mu_alpha, mu_delta, rad_vel,
            app_mag, abs_mag, color, size,
            hip_number,
            source_id,
            name_len,
            names   
        });

    }
}
