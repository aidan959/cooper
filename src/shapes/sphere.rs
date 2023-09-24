use super::super::math::{vec3::{Vec3,dot}, ray::Ray};
use super::detect::{HitRecord, Hittable};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}
impl Hittable for Sphere {
    fn hit(
        self: &mut Self,
        ray: &Ray,
        ray_tmin: f64,
        ray_tmax: f64,
        rec: &mut HitRecord,
    ) -> bool{
        let oc = ray.origin - self.center;
        let a = ray.direction.sqr_len();
        let half_b =  dot(oc, ray.direction);
        let c = oc.sqr_len() - self.radius*self.radius;
        
        let discriminant = half_b * half_b - a * c;
        
        if discriminant < 0. { return false; }
        
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b -sqrtd) / a;
        if root <= ray_tmin || ray_tmax <= root {
            root = (-half_b + sqrtd)/a ;
            if root <= ray_tmin || ray_tmax <= root{
                return false;
            }
        }
        
        rec.t = root;
        rec.p = ray.at(rec.t);
        let outward_normal: Vec3 = (rec.p -self.center) / self.radius;
        rec.set_face_normal(ray, outward_normal);
        rec.normal = (rec.p - self.center) / self.radius; 

        return true;
    }
}
