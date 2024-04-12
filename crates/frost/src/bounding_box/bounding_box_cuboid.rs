use glam::DMat4;

use crate::bounding_box::BoundingBox;
use crate::shapes::Cuboid;


impl Cuboid {

    #[inline]
    pub fn bounding_box(&self, pos: &DMat4) -> BoundingBox {
        let (_, _, translation) = pos.to_scale_rotation_translation();
        let center = translation.as_vec3();
        let ws_half_extents = pos.transform_vector3(self.half_extents.as_dvec3());
        println!("{}", ws_half_extents);
        BoundingBox::from_he(center, ws_half_extents.as_vec3())
    }

    #[inline]
    pub fn local_aabb(&self) -> BoundingBox {
        let half_extents = self.half_extents;

        BoundingBox::new(-half_extents, half_extents)
    }
}
