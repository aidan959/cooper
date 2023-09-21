// use:: {
//     io::*
// };
static X: u8 = 0;
static Y: u8 = 1;
static Z: u8 = 2;



pub struct Vec3{
    e: [f64; 3],
}

impl Vec3 {
    fn blank() -> Self {
        Vec3{e: [0.,0.,0.]}
    }
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 {e: [x, y ,z ]}
    }
    fn x(&self) -> f64 {
        self.e[X]
    }
    fn y(&self) -> f64 {
        self.e[Y]
    }
    fn x(&self) -> f64 {
        self.e[Z]
    }
    fn sqr_len(&self) -> f64 {
        // TODO: SPEED TEST THIS?e[X].pow(2) + e[Y].pow(2) + e[Z].pow(3)
        e[X]*e[X] + e[Y]*e[Y] + e[Z]*e[Z]
    }
    fn len(&self) -> f64 {
        self.sqr_len().sqrt()
    }
}
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::with_values
    }
}
