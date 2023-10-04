use crate::math::normalize;
use cgmath::Point3;
use cgmath::Point2;
pub const  PI : f32 = 3.14159265358979323846;


#[derive(Clone, Copy)]
pub struct Camera {
    last_look_position: Option<Point2<f32>>,
    sensitivity: f32,
    location: Point3<f32>,
    pitch: f32,
    yaw: f32,
    r: f32,
}


impl Camera {
    pub fn position(&self) -> Point3<f32> {
        self.location
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