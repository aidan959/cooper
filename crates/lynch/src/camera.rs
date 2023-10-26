use crate::math::cross_product;
use crate::math::normalize;
use cgmath::Point3;
use cgmath::Point2;
use crate::constants::PI;


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
    pub fn get_forward(&self) -> [f32; 3]{
        let forward_vector = [
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos()
        ];
        return normalize(forward_vector);
    } 
    pub fn move_forward(&mut self, amount: f32){
        
        let forward = self.get_forward();
        self.location[0] +=  forward[0] * amount;
        self.location[1] +=  forward[1] * amount;
        self.location[2] +=  forward[2] * amount;
    }
    pub fn move_sideways(&mut self, amount: f32){
        let forward = self.get_forward();

        let sideway = cross_product(forward, [0.,1.,0.]);
        
        self.location[0] +=  sideway[0] * amount;
        self.location[1] +=  sideway[1] * amount;
        self.location[2] +=  sideway[2] * amount;
    }
}

//orbital
//Point3::new(
//     self.r * self.phi.sin() * self.theta.sin(),
//     self.r * self.phi.cos(),
//     self.r * self.phi.sin() * self.theta.cos(),
// )

impl Camera {
    pub fn mouse_moved(&mut self, mouse_pos: [i32; 2]) {
        self.yaw -= mouse_pos[0] as f32 * self.sensitivity;
        self.pitch =  (self.pitch - (mouse_pos[1] as f32 * self.sensitivity)).clamp(-(PI / 2.) +0.01, PI / 2. -0.01);
    }
    pub fn get_look_toward(&mut self) -> Point3<f32> {
        let r = self.pitch.cos();
        
        let mut y = self.pitch.sin();
        let mut z = r * self.yaw.cos();
        let mut x = r * self.yaw.sin();
        x += self.location.x;
        y += self.location.y;
        z += self.location.z;

        Point3 { x: x, y: y, z: z }
        


    }
    pub fn forward(&mut self, r: f32) {
        self.r -= r;
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
