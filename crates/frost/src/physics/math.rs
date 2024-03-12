use glam::{Quat, Vec3,Mat3};

use super::{*,obb::CollisionPoint};

#[inline]
pub fn integrate_velocity(velocity: &mut Vec3, acceleration: Vec3, fixed_time: f32) {
    *velocity += acceleration * fixed_time;
}
#[inline]
pub fn integrate_position(transform: &mut Transform, velocity: Vec3, fixed_time: f32) {
    transform.position += velocity * fixed_time;
}

pub fn integrate_rigid_body(rigid_body: &mut RigidBody, fixed_time: f32) {
    if rigid_body.gravity {
        rigid_body.acceleration += Vec3::new(0., -9.81, 0.);
    }
    integrate_velocity(&mut rigid_body.velocity, rigid_body.acceleration, fixed_time);
    integrate_position(&mut rigid_body.transform, rigid_body.velocity, fixed_time);
    rigid_body.clear_accumulators();
}

pub fn calculate_velocity_change(velocity: Vec3, acceleration: Vec3, fixed_time: f32) -> Vec3 {
    velocity + acceleration * fixed_time
}

pub fn calculate_position_change(position: Vec3, velocity: Vec3, fixed_time: f32) -> Vec3 {
    position + velocity * fixed_time
}
fn calculate_relative_velocity(rigid_body: &RigidBody, rigid_body2: &RigidBody, relative_p1: Vec3, relative_p2: Vec3) -> Vec3 {
    rigid_body.velocity + relative_p1.cross(rigid_body.angular_velocity) - rigid_body2.velocity - relative_p2.cross(rigid_body2.angular_velocity)
}
pub fn handle_collision(rigid_body: &mut RigidBody, rigid_body2: &mut RigidBody, collision_point: &CollisionPoint) {
    // get relative velocity at collision point
    let relative_p1 = collision_point.point - rigid_body.transform.position;
    let relative_p2 = collision_point.point - rigid_body2.transform.position;
    

    let relative_velocity = calculate_relative_velocity(&rigid_body, &rigid_body2, relative_p1, relative_p2);
    
    // do we use min or max here?
    let e = (rigid_body.restitution + rigid_body2.restitution) / 2.0;
    
    let r1_cross_n = relative_p1.cross(collision_point.normal);
    let r2_cross_n = relative_p2.cross(collision_point.normal);

    let inv_mass_sum = rigid_body.inverse_mass + rigid_body2.inverse_mass +
                    r1_cross_n.dot(rigid_body.inverse_inertia_tensor * r1_cross_n ) +
                    r2_cross_n.dot(rigid_body2.inverse_inertia_tensor * r2_cross_n );
    
    let j = (-(1.0 + e) * relative_velocity.dot(collision_point.normal)) / inv_mass_sum;

    let impulse = collision_point.normal * j;

    rigid_body.velocity += impulse * rigid_body.inverse_mass;
    rigid_body.angular_velocity += rigid_body.inverse_inertia_tensor * r1_cross_n.cross(impulse);
    rigid_body2.velocity -= impulse * rigid_body2.inverse_mass;
    rigid_body.angular_velocity -= rigid_body2.inverse_inertia_tensor * r1_cross_n.cross(impulse);

}

pub struct RigidBody{
    pub inverse_mass: f32,
    pub transform: Transform,
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub angular_velocity:Vec3,
    pub inverse_inertia_tensor: Mat3,
    pub gravity: bool,
    pub restitution: f32
}


impl RigidBody {
    fn apply_force(&mut self, force: Vec3, point: Vec3) {
        // Implementation
    }

    fn apply_torque(&mut self, torque: Vec3) {
        // Implementation
    }

    fn clear_accumulators(&mut self) {
        // Implementation
    }
}

pub struct Transform{
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            inverse_mass : 0.0,
            transform : Default::default(),
            acceleration : Default::default(),
            velocity : Default::default(),
            angular_velocity : Default::default(),
            inverse_inertia_tensor : Default::default(),
            gravity : Default::default(),
            restitution  : 0.0,
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