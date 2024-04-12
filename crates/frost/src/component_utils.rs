use crate::ComponentVec;
use std::{any::TypeId, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::RwLock};

pub(crate) fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>()
        .unwrap()
        .get_mut()
        .unwrap()
}

pub(crate) fn calculate_pack_id(types: &[TypeId]) -> u64 {
    let mut s = DefaultHasher::new();
    types.hash(&mut s);
    s.finish()
}