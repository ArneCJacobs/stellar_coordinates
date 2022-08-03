#[allow(dead_code)]
pub const METADATA_FILE: &'static str = "metadata.bin";
#[allow(dead_code)]
pub const GAIASKY_INTERNAL_UNIT: f64 = 1e10; // meter
#[allow(dead_code)]
pub const ASTRONOMICAL_UNIT: f64 = 1.495978707e11; // meter
pub const PARSEC: f64 = 30_856_775_814_913_673.0; // meter
pub const LIGHT_YEAR: f64 = 5.88e12;
pub const DATA_SCALE: f64 = 1.0 / 10.0; 
pub const GAIASKY_INTERNAL_UNIT_TO_PARSEC_SCALED: f64 = GAIASKY_INTERNAL_UNIT * DATA_SCALE / PARSEC;
pub const GITPS: f64 = GAIASKY_INTERNAL_UNIT_TO_PARSEC_SCALED;

// https://gaia.ari.uni-heidelberg.de/gaiasky/docs/master/Internal-reference-system.html#reference-system
#[allow(dead_code)]
pub fn gaiasky_to_cartesian<T, A, B>(a: A) -> B
    where 
        A: Into<[T; 3]>, 
        B: From<[T; 3]>,
{
    let [x, y, z] = a.into();
    return B::from([z, x, y]);
}
