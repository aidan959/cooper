use glam::{Quat, Vec3,Mat3, const_vec3};

use super::{*,obb::CollisionPoint};
pub use math::{
    Transform,
    handle_collision,
    calculate_velocity_change,
    calculate_position_change,
    RigidBody};

pub mod math;

pub const GRAVITY: Vec3 = const_vec3!([0.0, -9.8, 0.0]);
