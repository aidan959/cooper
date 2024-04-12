use glam::Vec3;

pub trait BoundingVolume {

    fn center(&self) -> Vec3;

    fn intersects(&self, _: &Self) -> bool;

    fn contains(&self, _: &Self) -> bool;

}
