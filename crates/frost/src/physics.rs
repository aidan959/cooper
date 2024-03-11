use glam::{Quat, Vec3};

use super::*;

pub struct Velocity {
    pub velocity: Vec3
}
pub struct RigidBody{
    pub mass: f32,
    pub drag: f32,
    pub angular_drag:f32,
    pub gravity: bool,
    pub velocity: Velocity
}
pub struct HitBox {

}
pub struct Transform{
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            velocity: Default::default()
        }
    }
}
impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass:1.,
            drag: 0.,
            angular_drag: 0.05,
            gravity: true,
            velocity: Default::default() , 
        }
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Default::default()
        }
    }
}