use glam::Vec3;

pub struct Mass{
    pub l_center_o_mass: Vec3,
    pub inv_mass: f32,
}

impl Mass {
    pub fn mass(&self) -> f32 {
        1.0/self.inv_mass
    }
}