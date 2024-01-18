pub mod entity;
pub mod world;
mod input;

pub use input::Input; 

pub(crate) type EntityId = u32;
pub(crate) type Generation = EntityId;
pub(crate) type PackId = u64;

#[derive(Clone, Copy)]
pub(crate) struct EntityMeta {
    pub(crate) generation: Generation,
    pub(crate) location: EntityLocation,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityLocation {
    archetype_index: EntityId,
    index_in_archetype: EntityId,
}