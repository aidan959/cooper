
pub const INFINITY : f64 = f64::INFINITY;
pub const PI : f64 = 3.1415926535897932385;

#[inline]
pub fn degrees_to_radians(degrees: f64) {
    degrees * PI / 180.;
}

#[inline]
pub fn radians_to_degrees(radians: f64) {
    radians * 180. / PI;
}