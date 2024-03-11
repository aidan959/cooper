use glam::Vec3;

use super::{X,Y,Z};

pub trait Vec3Tools {
    fn max_element_index (&self) -> usize;
    fn min_element_index (&self) -> usize;
    fn mid_point(p1: &Self, p2: &Self) -> Self;
}

impl Vec3Tools for Vec3 {
    fn max_element_index (&self) -> usize{
        let mut max = X;

        if self.y > self.x {
            max = Y;            
        }
        if self.z > self[max] {
            max = Z;
        }
        max
    }
    fn min_element_index (&self) -> usize{
        let mut min = X;

        if self.y < self.x {
            min = Y;            
        }
        if self.z < self[min] {
            min = Z;
        }
        min
    }
    fn mid_point(p1: &Self, p2: &Self) -> Self {
        (*p1 + *p2) * 0.5
    }
}