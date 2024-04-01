use dolly::prelude::*;
use glam::{Mat3, Mat4, Quat, Vec3};

use frost::Input;

use crate::WindowSize;

pub struct Camera {
    camera_rig: CameraRig,
    fov_degrees: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    speed: f32,
}
impl Camera {
    pub(crate) fn new(
        pos: Vec3,
        target: Vec3,
        fov_degrees: f32,
        window_size: WindowSize,
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
            aspect_ratio: get_aspect_ratio(window_size) as f32,
            z_near,
            z_far,
            speed,
        }
    }

    pub fn get_lookat_rotation(pos: Vec3, target: Vec3) -> Quat {
        (target - pos)
            .try_normalize() // can return none
            .and_then(|forward| {
                let right = forward.cross(Vec3::Y).try_normalize()?;
                let up = right.cross(forward);
                Some(Quat::from_mat3(&Mat3::from_cols(right, up, -forward)))
            })
            .unwrap_or_default()
    }

    pub fn update(&mut self, input: &Input, delta: f32) -> bool {
        let transform = self.camera_rig.final_transform;

        let mut movement = Vec3::new(0.0, 0.0, 0.0);
        if input.key_down(winit::event::VirtualKeyCode::W) {
            movement += self.speed * transform.forward();
        }
        if input.key_down(winit::event::VirtualKeyCode::S) {
            movement -= self.speed * transform.forward();
        }
        if input.key_down(winit::event::VirtualKeyCode::A) {
            movement -= self.speed * transform.right();
        }
        if input.key_down(winit::event::VirtualKeyCode::D) {
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
        self.camera_rig.update(delta);
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

    pub fn get_projection(&self) -> Mat4 {
        glam::Mat4::perspective_rh(
            f32::to_radians(self.fov_degrees),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn get_position(&self) -> Vec3 {
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
    pub fn builder() -> CameraBuilder<Option<f32>> {
        CameraBuilder::new()
    }
}


fn get_aspect_ratio(window_size : WindowSize) -> f64{
    window_size.0 / window_size.1
}




pub struct CameraBuilder<AR> {
    camera_rig: Option<CameraRig>,
    aspect_ratio: AR,
    fov_degrees: f32,
    z_near: f32,
    z_far: f32,
    speed: f32,
    position: Vec3,
    rotation: Quat
}
const DEFAULT_FOV: f32 = 90.0;
const DEFAULT_NEAR: f32 = 0.01;
const DEFAULT_FAR: f32 = 1000.0;
const DEFAULT_SPEED: f32 = 0.2;


impl CameraBuilder<Option<f32>> {
    pub fn new() -> Self {
        CameraBuilder {
            camera_rig: None,
            fov_degrees: DEFAULT_FOV,
            aspect_ratio: None,
            z_near: DEFAULT_NEAR,
            z_far: DEFAULT_FAR,
            speed: DEFAULT_SPEED,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY
        }
    }

    pub fn camera_rig(mut self, camera_rig: CameraRig) -> Self {
        self.camera_rig = Some(camera_rig);
        self
    }

    pub fn fov_degrees(mut self, fov_degrees: f32) -> Self {
        self.fov_degrees = fov_degrees;
        self
    }

    pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    pub fn aspect_ratio_from_window(mut self, window_size : WindowSize) -> Self {
        self.aspect_ratio = Some(get_aspect_ratio(window_size) as f32);
        self
    }

    pub fn z_near(mut self, z_near: f32) -> Self {
        self.z_near = z_near;
        self
    }

    pub fn z_far(mut self, z_far: f32) -> Self {
        self.z_far = z_far;
        self
    }
    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }
    pub fn target(mut self, target: Vec3) -> Self {
        self.rotation = Camera::get_lookat_rotation(self.position, target);
        self
    }
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    pub fn build(self) -> Camera {
        let camera_rig = self.camera_rig.unwrap_or_else(||{CameraRig::builder()
            .with(Position::new(self.position))
            .with(YawPitch::new().rotation_quat(self.rotation))
            .with(Smooth::new_position_rotation(0.9, 0.9))
            .build()});
        let fov_degrees = self.fov_degrees;
        let aspect_ratio = self.aspect_ratio.expect("Aspect ratio is a required property for camera builder.");
        let z_near = self.z_near;
        let z_far = self.z_far;
        let speed = self.speed;

        Camera {
            camera_rig,
            fov_degrees,
            aspect_ratio,
            z_near,
            z_far,
            speed,
        }
    }
}

impl Camera {
    
}
