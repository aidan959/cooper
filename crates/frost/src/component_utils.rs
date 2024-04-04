use crate::PackId;
use std::{any::TypeId, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::RwLock};

pub(super) fn calculate_pack_id(types: &[TypeId]) -> PackId {
    let mut s = <DefaultHasher as std::default::Default>::default();
    types.hash(&mut s);
    s.finish()
}