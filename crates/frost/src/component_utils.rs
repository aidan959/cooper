use crate::ComponentVec;
use std::{any::TypeId, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::RwLock};

pub(super) fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<T> {
    Result::unwrap({
        let this = c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>();
        match this {
            Some(val) => val,
            None => panic!("called `unwrap on non existent empty value"),
        }
    }.get_mut())
}

pub(super) fn calculate_pack_id(types: &[TypeId]) -> u64 {
    let mut s = <DefaultHasher as std::default::Default>::default();
    types.hash(&mut s);
    s.finish()
}