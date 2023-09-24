use super::vec3::Vec3;
use std::{
    fmt::{Display, Formatter, Result},
    ops::{Mul, Add}
};
#[derive(Clone,Copy,PartialEq)]

pub struct Colour {
    pub vector: Vec3,
}

impl Colour {
    pub const BLACK : Colour = Colour{vector: Vec3{x:255.,y:255.,z:255.}};
    pub const WHITE : Colour = Colour{vector: Vec3{x:0.,y:0.,z:0.}};

    pub fn new(x: u16, y: u16, z: u16) -> Self {
        Colour {
            vector: Vec3 {
                x: (x % 256) as f64,
                y: (y % 256) as f64,
                z: (z % 256) as f64,
            },
        }
    }
    pub fn new_fraction(x: f64, y: f64, z: f64) -> Self {
        Colour {
            vector: Vec3 {
                x: 255.999 * x,
                y: 255.999 * y,
                z: 255.999 * z,
            },
        }
    }
    pub fn new_from_vector(vector :Vec3) -> Self {
        Colour {
            vector: vector
            }
        
    }
}
impl Mul for Colour {
    type Output = Colour;
    fn mul(self, rhs: Colour) -> Self {
        Colour::new_from_vector(self.vector * rhs.vector )
    }
}
impl Mul<f64> for Colour {
    type Output = Colour;
    fn mul(self, rhs: f64) -> Self {
        Colour::new_from_vector(self.vector * rhs )
    }
}
impl Mul<Colour> for f64 {
    type Output = Colour;
    fn mul(self, rhs: Colour) -> Colour {
        Colour::new_from_vector(rhs.vector * self  )
    }
}
impl Add for Colour {
    type Output = Colour;
    fn add(self, rhs: Colour) -> Self {
        Colour::new_from_vector(self.vector+ rhs.vector )
    }
}
impl Display for Colour {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{} {} {}",
            self.vector.x as u8,
            self.vector.y as u8,
            self.vector.z as u8
        )
    }
}
