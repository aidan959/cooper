use crate::math::normalize;
use cgmath::Point3;
use cgmath::Point2;
pub const  PI : f32 = 3.14159265358979323846;
use dolly::prelude::*;
use glam::{Mat3, Mat4, Quat, Vec3};

#[derive(Clone, Copy)]
pub struct Camera {
    camera_rig: CameraRig,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    sensitivity: f32,
}


impl Camera {
    pub fn new(
        pos: Vec3,
        target: Vec3,
        fov_degrees: f32,
        aspect_ratio: f64,
        z_near: f32,
        z_far: f32,
        speed: f32,
    ) -> Camera {
        let rotation = Self::get_lookat_rotation(pos, target);

        let camera_rig = CameraRig::builder()
            .with(Position::new(pos))
            .with(YawPitch::new().rotation_quat(rotation))
            .with(Smooth::new_position_rotation(0.9, 0.9))
            .build();

        Camera {
            camera_rig,
            fov_degrees,
            aspect_ratio: aspect_ratio as f32,
            z_near,
            z_far,
            speed,
        }
    }
    pub fn position(&self) -> Vec3 {
        self.camera_rig.final_transform.position
    }

    pub fn get_lookat_rotation(pos: Vec3, target: Vec3) -> Quat {
        (target - pos)
            .try_normalize()
            .and_then(|forward| {
                let right = forward.cross(Vec3::Y).try_normalize()?;
                let up = right.cross(forward);
                Some(Quat::from_mat3(&Mat3::from_cols(right, up, -forward)))
            })
            .unwrap_or_default()
    }
    pub fn forward(&mut self, r: f32) {
        self.r -= r;
    }
    pub fn update(&mut self, input: &Input) -> bool {
        let transform = self.camera_rig.final_transform;
        // TODO abstract to ecs / get inpputs from ecs
        // two ideas: One -> ECS passes input and this is just handled
        // in "update" blocks by anything that has Input struct
        // Two -> Update is handled as system in and of itself and
        // and we just 
        let mut movement = Vec3::new(0.0, 0.0, 0.0);
        if input.key_down(winit::keyboard::KeyCode::KeyW) {
            movement += self.speed * transform.forward();
        }
        if input.key_down(winit::keyboard::KeyCode::KeyS) {
            movement -= self.speed * transform.forward();
        }
        if input.key_down(winit::keyboard::KeyCode::KeyA) {
            movement -= self.speed * transform.right();
        }
        if input.key_down(winit::keyboard::KeyCode::KeyD) {
            movement += self.speed * transform.right();
        }

        self.camera_rig.driver_mut::<Position>().translate(movement);

        let mut view_changed = false;
        if input.right_mouse_down {
            self.camera_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-0.3 * input.mouse_delta.x, -0.3 * input.mouse_delta.y);
            view_changed = true;
        }
        self.camera_rig.update(1.0); 
        movement != Vec3::new(0.0, 0.0, 0.0) || view_changed
    }
    pub fn get_view(&self) -> Mat4 {
        let transform = self.camera_rig.final_transform;

        glam::Mat4::look_at_rh(
            transform.position,
            transform.position + transform.forward(),
            transform.up(),
        )
    }
    // glamafied projection
    pub fn get_projection(&self) -> Mat4 {
        glam::Mat4::perspective_rh(
            f32::to_radians(self.fov_degrees),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }
    #pub fn get_position(&self) -> Vec3 {
        self.camera_rig.final_transform.position
    }

    pub fn get_forward(&self) -> Vec3 {
        self.camera_rig.final_transform.forward()
    }

    pub fn set_position_target(&mut self, position: Vec3, target: Vec3) {
        self.camera_rig.driver_mut::<Position>().position = position;

        let rotation = Self::get_lookat_rotation(position, target);
        self.camera_rig
            .driver_mut::<YawPitch>()
            .set_rotation_quat(rotation);
    }

    pub(crate) fn get_near_plane(&self) -> f32 {
        self.z_near
    }

    pub(crate) fn get_far_plane(&self) -> f32 {
        self.z_far
    }
}


impl Default for Camera {
    fn default() -> Self {
        Camera {
            last_look_position: None,
            sensitivity: 0.01,
            location: Point3::new(2., 2., 2.),
            pitch: 0.,
            yaw: 0.,
            r: 0.
        } 
    }
}