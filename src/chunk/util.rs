#[allow(dead_code)]
pub const METADATA_FILE: &'static str = "metadata.bin";
#[allow(dead_code)]
pub const GAIASKY_INTERNAL_UNIT: f64 = 1e10; // meter
#[allow(dead_code)]
pub const ASTRONOMICAL_UNIT: f64 = 1.495978707e11; // meter
#[allow(dead_code)]
pub const PARSEC: f64 = 30_856_775_814_913_673.0; // meter
pub const DATA_SCALE: f64 = 1.0 / 10.0; 
#[allow(dead_code)]
pub const GAIASKY_INTERNAL_UNIT_TO_PARSEC: f64 = GAIASKY_INTERNAL_UNIT * DATA_SCALE / PARSEC;
#[allow(dead_code)]
pub const GITP: f64 = GAIASKY_INTERNAL_UNIT_TO_PARSEC;
