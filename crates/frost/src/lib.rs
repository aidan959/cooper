pub mod entity;
pub mod world;
mod input;

pub use input::Input; 

pub(crate) type EntityId = u32;
pub(crate) type Generation = EntityId;
pub(crate) type PackId = u64;
pub struct World {
    pub(crate) entities: Vec<EntityMeta>,
    available_entities: Vec<EntityId>,
    pack_id_to_archetype: HashMap<PackId, usize>,
    archetypes: Vec<Archetype>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entities: Vec::new(),
            pack_id_to_archetype: HashMap::new(),
            available_entities: Vec::new(),
        }
    }
    pub fn new_entity(&mut self, components: impl ComponentPack) -> Result<Entity, WorldFull> {
        let (index, generation) = if let Some(index) = self.available_entities.pop() {
            let (generation, _) = self.entities[index as usize].generation.overflowing_add(1);
            (index, generation)
        } else {
            self.entities.push(EntityMeta::null());

            match self.entities.len() >= EntityId::MAX as usize {
                true => return Err(WorldFull::new()),
                false => (),
            }
            ((self.entities.len() - 1) as EntityId, 0)
        };

        self.entities[index as usize] = EntityMeta {
            location: components.spawn(self, index),
            generation: generation,
        };
        Ok(Entity { index, generation })
    }
    #[inline]
    pub fn retrieve_single<T: 'static>(&self) -> Result<Single<T>, RetrieveError> {
        <&T>::retrieve(self)
    }
    #[inline]
    pub fn retrieve_single_mut<T: 'static>(&self) -> Result<SingleMut<T>, RetrieveError> {
        <&mut T>::retrieve(self)
    }
}
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

pub trait ComponentPack: 'static + Send + Sync {
    fn new_archetype(&self) -> Archetype;
    fn spawn(self, world: &mut World, entity_index: EntityId) -> EntityLocation;
}

impl<A: 'static + Send + Sync> ComponentPack for (A,) {
    fn new_archetype(&self) -> Archetype {
        let mut components = vec![ComponentStore::new::<A>()];
        components.sort_unstable_by(|a, b| a.type_id.cmp(&b.type_id));
        Archetype {
            components,
            entities: Vec::new(),
        }
    }
    fn spawn(self, world: &mut World, entity_index: EntityId) -> EntityLocation {
        let mut types = [(0, TypeId::of::<A>())];
        types.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        debug_assert!(
            types.windows(2).all(|x| x[0].1 != x[1].1),
            "`ComponentPack`s cannot contain duplicate components."
        );
        let mut order = [0; 1];
        (0..order.len()).for_each(|i| {
            order[types[i].0] = i;
        });
        let types = [types[0].1];
        let bundle_id = calculate_pack_id(&types);
        let archetype_index = if let Some(archetype) = world.pack_id_to_archetype.get(&bundle_id) {
            *archetype
        } else {
            let index = world.archetypes.len();
            world.pack_id_to_archetype.insert(bundle_id, index);
            world.archetypes.push(self.new_archetype());
            index
        };
        world.archetypes[archetype_index]
            .entities
            .push(entity_index);
        world.archetypes[archetype_index].push(order[0], self.0);
        EntityLocation {
            archetype_index: archetype_index as EntityId,
            index_in_archetype: (world.archetypes[archetype_index].len() - 1) as EntityId,
        }
    }
}

pub(crate) struct ComponentStore {
    pub(crate) type_id: TypeId,
    data: Box<dyn ComponentVec + Send + Sync>,
}

impl ComponentStore {
    pub fn new<T: 'static + Send + Sync>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: Box::new(RwLock::new(Vec::<T>::new())),
        }
    }
    pub fn new_same_type(&self) -> Self {
        Self {
            type_id: self.type_id,
            data: self.data.new_same_type(),
        }
    }
}


impl<T: Component> ComponentVec for RwLock<Vec<T>> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&mut self) -> usize {
        self.get_mut().unwrap().len()
    }

    fn swap_remove(&mut self, index: EntityId) {
        self.get_mut().unwrap().swap_remove(index as usize);
    }

    fn migrate(&mut self, entity_index: EntityId, other_component_vec: &mut dyn ComponentVec) {
        let data: T = self.get_mut().unwrap().swap_remove(entity_index as usize);
        Result::unwrap(
            {
                let this = other_component_vec
                    .to_any_mut()
                    .downcast_mut::<RwLock<Vec<T>>>();
                match this {
                    Some(val) => val,
                    None => panic!("called `unwrap on non existing empty value"),
                }
            }
            .get_mut(),
        )
        .push(data);
    }

    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync> {
        Box::new(RwLock::new(Vec::<T>::new()))
    }
}
