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

pub trait Component: Sync + Send + 'static {}
impl<T: Sync + Send + 'static> Component for T {}

trait ComponentVec: Sync + Send {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn len(&mut self) -> usize;
    fn swap_remove(&mut self, index: EntityId);
    fn migrate(&mut self, entity_index: EntityId, other_archetype: &mut dyn ComponentVec);
    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync>;
}
pub struct Archetype {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) components: Vec<Component>,
}

impl EntityLocation {
    fn null() -> Self {
        Self {
            archetype_index: 0,
            index_in_archetype: 0,
        }
    }
    fn new(archetype_index: EntityId, index_in_archetype: EntityId) -> Self {
        Self {
            archetype_index,
            index_in_archetype,
        }
    }
}

impl EntityMeta {
    fn null() -> Self {
        EntityMeta {
            generation: 0,
            location: EntityLocation::null(),
        }
    }
    fn archetype_index(self) -> EntityId {
        self.location.archetype_index
    }
    fn index_in_archetype(self) -> EntityId {
        self.location.index_in_archetype
    }
}
