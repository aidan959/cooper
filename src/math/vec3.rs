use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, Neg, Sub, Mul, Div, AddAssign, MulAssign, DivAssign}, vec,
};

// static X: i32 = 0;
// static Y: i32 = 1;
// static Z: i32 = 2;/
#[derive(Clone,Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64, 
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl Vec3 {
    pub fn blank() -> Self {
        Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x: x, y: y, z: z }
    }
    /*fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
    fn z(&self) -> f64 {
        self.z
    }*/
    pub fn dot(&self, other: Vec3) -> f64 {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }
    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn sqr_len(&self) -> f64 {
        // TODO: SPEED TEST THIS?e[X].pow(2) + e[Y].pow(2) + e[Z].pow(3)
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn len(&self) -> f64 {
        self.sqr_len().sqrt()
    }
    pub fn unit_vector(&self) -> Vec3 {
        self / self.len()
    }
    
}
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl Sub<f64> for Vec3 {
    type Output = Vec3;
    fn sub(self, val: f64) -> Vec3 {
        Vec3 {
            x: self.x - val,
            y: self.y - val,
            z: self.z - val,
        }
    }
}
impl Sub<Vec3> for f64 {
    type Output = Vec3;
    fn sub(self, val: Vec3) -> Vec3 {
        Vec3 {
            x: self - val.x,
            y: self - val.y,
            z: self - val.z,
        }
    }
}

/* /////TODO: investigate
impl Add for Vec3 {
    type Output = Vec3;

    fn add(&mut self, other: Vec3) -> &mut Vec3 {
        self.x + other.x;
        self.y + other.y;
        self.z + other.z;
        self
    }
}*/

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<Vec3> for &Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<&Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn add(self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<f64> for Vec3 {
    type Output = Vec3;
    fn add(self, val: f64) -> Vec3 {
        Vec3 {
            x: self.x + val,
            y: self.y + val,
            z: self.z + val,
        }
    }
}
impl Add<f64> for &Vec3 {
    type Output = Vec3;
    fn add(self, val: f64) -> Vec3 {
        Vec3 {
            x: self.x + val,
            y: self.y + val,
            z: self.z + val,
        }
    }
}

impl Add<Vec3> for f64 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self + other.x,
            y: self + other.y,
            z: self + other.z,
        }
    }
}

impl Add<&Vec3> for f64 {
    type Output = Vec3;

    fn add(self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self + other.x,
            y: self + other.y,
            z: self + other.z,
        }
    }
}

impl Add<Vec3> for &f64 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self + other.x,
            y: self + other.y,
            z: self + other.z,
        }
    }
}

impl Add<&Vec3> for &f64 {
    type Output = Vec3;

    fn add(self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self + other.x,
            y: self + other.y,
            z: self + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}
impl AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, other: &Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl AddAssign<f64> for Vec3 {
    fn add_assign(&mut self, val: f64) {
        self.x += val;
        self.y += val;
        self.z += val;
    }
}

impl Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}
impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, val: f64) -> Vec3 {
        Vec3 {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val,
        }
    }
}
impl Mul<f64> for &Vec3 {
    type Output = Vec3;
    fn mul(self, val: f64) -> Vec3 {
        Vec3{x:self.x * val, y:self.y * val, z:self.z * val}
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, val: Vec3) -> Vec3 {
        Vec3 {
            x: val.x * self,
            y: val.y * self,
            z: val.z * self,
        }
    }
}
impl Mul<&Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, val: &Vec3) -> Vec3 {
        Vec3 {
            x: val.x * self,
            y: val.y * self,
            z: val.z * self,
        }
    }
}
impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, val: f64) -> Vec3 {
        (1./val) * self
    }
}
impl Div<f64> for &Vec3 {
    type Output = Vec3;
    fn div(self, val: f64) -> Vec3 {
        (1./val) * self
    }
}
impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, val: f64){
        *self *= 1./val
    }
}
impl DivAssign<f64> for &mut Vec3 {
    fn div_assign(&mut self, val: f64) {
        *self *= 1./val 
    }
}
impl MulAssign for Vec3 {
    fn mul_assign(&mut self, other: Vec3){
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    } 
}
impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, val: f64){
        self.x *= val;
        self.y *= val;
        self.z *= val;
    } 
}
impl MulAssign<f64> for &mut Vec3 {
    fn mul_assign(&mut self, val: f64){
        self.x *= val;
        self.y *= val;
        self.z *= val;
    } 
}
impl PartialEq for Vec3 {
    fn eq(&self, val : &Vec3) -> bool {
        self.x == val.x && self.y == val.y && self.z == val.z 
    }
}

// pub fn dot(one: &Vec3, other: &Vec3) -> f64 {
//     return one.x * other.x + one.y * other.y + one.z * other.z;
// }
pub fn dot(one: Vec3, other: Vec3) -> f64 {
    return one.x * other.x + one.y * other.y + one.z * other.z;
}
pub fn cross(one: Vec3, other: Vec3) -> Vec3 {
    Vec3 {
        x: one.y * other.z - other.z * other.y,
        y: one.z * other.x - other.x * other.z,
        z: one.x * other.y - other.y * other.x,
    }
}
pub fn unit_vector(vector: Vec3) -> Vec3{
    vector / vector.len()
}