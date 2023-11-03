use super::super::math::{
    ray::Ray,
    vec3::{dot, Vec3},
};
use std::vec::Vec;
pub struct HitRecord {
    pub p: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
}
impl HitRecord {
    pub fn blank() -> Self {
        HitRecord { p: Vec3::blank(), normal: Vec3::blank(), t: 0., front_face: false }
    }
    pub fn set_face_normal(self: &mut Self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = dot(ray.direction, outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        }
    }
}
pub trait Hittable {
    fn hit(self: &mut Self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &&mut HitRecord) -> bool;
}

pub struct EntityList {
    objects: Vec<Box<dyn Hittable>>,
}
impl Hittable for EntityList {
    fn hit(
        self: &mut Self,
        ray: &Ray,
        ray_tmin: f64,
        ray_tmax: f64,
        hit_record: &mut HitRecord,
    ) -> bool {
        let default_p = Vec3::blank();
        let default_normal= Vec3::blank();

        let mut temp_record:  HitRecord = HitRecord::blank();

        let mut hit: bool = false;
        let mut closest = ray_tmax;

        for object in &mut self.objects {
            if (object.hit(ray, ray_tmin, closest, &&mut temp_record)) {
                hit = true;
                closest = temp_record.t;
                *hit_record = &mut temp_record
            }
        }
        return hit;
    }
}

impl EntityList {
    pub fn new(object: Box<dyn Hittable>) -> Self {
        EntityList {
            objects: vec![object],
        }
    }
    pub fn clear(self: &mut Self) {
        self.objects.clear();
    }

    pub fn add(self: &mut Self, object: Box<dyn Hittable>) -> &Vec<Box<dyn Hittable>> {
        self.objects.push(object);
        return &self.objects;
    }
}
