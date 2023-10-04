use cgmath::prelude::*;
use cgmath::{BaseFloat, Matrix4, Rad};

pub fn normalize(vector : [f32; 3]) -> [f32; 3]{
    let length = (vector[0].powf(2.) + vector[1].powf(2.) + vector[2].powf(2.)).sqrt();
    return [
        vector[0]/length,
        vector[1]/length,
        vector[2]/length
    ]
}

pub fn cross_product(a : [f32;3], b : [f32;3]) -> [f32; 3]{
    return [a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0]]
}

/// Clamp `value` between `min` and `max`.
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    let value = if value > max { max } else { value };
    if value < min {
        min
    } else {
        value
    }
}