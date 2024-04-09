use glam::{Vec3, const_vec3};

use super::*;
pub use math::{
    Transform,
    handle_collision,
    calculate_velocity_change,
    calculate_position_change,
    RigidBody};

pub mod math;

pub const GRAVITY: Vec3 = const_vec3!([0.0, -0.98, 0.0]);
