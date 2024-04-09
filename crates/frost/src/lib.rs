pub mod entity;
pub mod math;
pub mod obb;
pub mod shapes;


mod input;
mod iter;
pub mod physics;
mod utils;
use utils::retrieve_two_mutable;
mod errors;
mod search;
mod system;
use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::RwLock,
};
use std::collections::hash_map::DefaultHasher;

pub use crate::{Retrieve, RetrieveError, SearchParameters, SearchRetrieve, Single, SingleMut};
pub use input::Input;
pub(crate) type EntityId = u32;
pub(crate) type Generation = EntityId;
pub(crate) type PackId = u64;
pub use iter::*;
pub use physics::*;
pub use search::Search;
pub use search::*;

pub use errors::*;
pub use system::*;
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

    #[doc = "Search from the world.
```
use frost::*;
let mut world = World::new();
let search = world.search<(&bool, &String)>();
```"]
    pub fn search<'world, T>(&'world self) -> Result<Search<T>, RetrieveError>
    where
        T: SearchParameters,
    {
        let retrieve = SearchRetrieve::<T>::retrieve(self);

        match retrieve {
            Ok(mut retrieve) => Ok(retrieve.take().unwrap()),
            Err(e) => Err(e),
        }
    }
    pub fn add_component<T>(&mut self, entity: Entity, t: T) -> Result<(), EntityNotFound>
    where
        T: 'static + Send + Sync,
    {
        let entity_meta = self.entities[entity.index as usize];

        if entity_meta.generation != entity.generation {
            return Err(EntityNotFound::new_with_value(entity.index));
        }
        let type_id = TypeId::of::<T>();

        let current_archetype = &self.archetypes[entity_meta.archetype_index() as usize];

        let mut type_ids: Vec<TypeId> = current_archetype
            .components
            .iter()
            .map(|c| c.type_id)
            .collect();

        let type_index = type_ids.binary_search(&type_id);

        if let Ok(insert_index) = type_index {
            let archetype = &mut self.archetypes[entity_meta.archetype_index() as usize];

            archetype.replace_component(insert_index, entity_meta.index_in_archetype(), t);
        } else {
            let insert_index = type_index.unwrap_or_else(|err| err);
            type_ids.insert(insert_index, type_id);
            let pack_id = calculate_pack_id(&type_ids);

            let new_archetype_index: usize = match self.pack_id_to_archetype.get(&pack_id) {
                Some(index) => *index,
                None => {
                    let mut archetype = Archetype::new();
                    current_archetype
                        .components
                        .iter()
                        .for_each(|c| archetype.components.push(c.new_same_type()));

                    let new_index = self.archetypes.len();
                    self.pack_id_to_archetype.insert(pack_id, new_index);

                    self.archetypes.push(archetype);

                    new_index
                }
            };

            let (old_archetype, new_archetype): (&mut Archetype, &mut Archetype) =
                retrieve_two_mutable(
                    &mut self.archetypes,
                    entity_meta.archetype_index() as usize,
                    new_archetype_index,
                );
            match old_archetype.entities.last() {
                Some(last) => {
                    self.entities[*last as usize].location = entity_meta.location;
                }
                _ => (),
            }
            self.entities[entity.index as usize].location = EntityLocation::new(
                new_archetype_index as EntityId,
                new_archetype.len() as EntityId,
            );

            for i in 0..insert_index {
                old_archetype.migrate_component(
                    i,
                    entity_meta.index_in_archetype(),
                    new_archetype,
                    i,
                );
            }

            new_archetype.push(insert_index, t);

            let components_in_archetype = old_archetype.components.len();

            for i in insert_index..components_in_archetype {
                old_archetype.migrate_component(
                    i,
                    entity_meta.index_in_archetype(),
                    new_archetype,
                    i.overflowing_add(1).0,
                );
            }

            old_archetype
                .entities
                .swap_remove(entity_meta.index_in_archetype() as usize);
            new_archetype.entities.push(entity.index);
        }

        Ok(())
    }
}

pub struct Archetype {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) components: Vec<ComponentStore>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: Vec::new(),
        }
    }
    pub(crate) fn retrieve<T: 'static>(&self, index: usize) -> &RwLock<Vec<T>> {
        let downcast_ref = self.components[index]
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>();
        downcast_ref.unwrap()
    }

    fn remove_entity(&mut self, index: EntityId) -> EntityId {
        for c in self.components.iter_mut() {
            c.data.swap_remove(index)
        }

        let moved = *self.entities.last().unwrap();
        self.entities.swap_remove(index as usize);
        moved
    }
    fn mutable_component_store<T: 'static>(&mut self, component_index: usize) -> &mut Vec<T> {
        Result::unwrap({
            let this = self.components[component_index].data
            .to_any_mut()
            .downcast_mut::<RwLock<Vec<T>>>();
            match this {
                Some(val) => val,
                None => panic!("called `unwrap on non existent empty value"),
            }
        }.get_mut())
    }

    fn replace_component<T: 'static>(&mut self, component_index: usize, index: EntityId, t: T) {
        self.mutable_component_store(component_index)[index as usize] = t;
    }

    fn push<T: 'static>(&mut self, component_index: usize, t: T) {
        self.mutable_component_store(component_index).push(t)
    }

    pub fn retrieve_component_mut<T: 'static>(
        &mut self,
        index: EntityId,
    ) -> Result<&mut T, ComponentNotInEntity> {
        let type_id = TypeId::of::<T>();
        let mut component_index = None;
        for (i, c) in self.components.iter().enumerate() {
            if c.type_id == type_id {
                component_index = Some(i);
                break;
            }
        }

        if let Some(component_index) = component_index {
            Ok(&mut self.mutable_component_store(component_index)[index as usize])
        } else {
            Err(ComponentNotInEntity::new_with_value::<T>(index))
        }
    }

    fn migrate_component(
        &mut self,
        component_index: usize,
        entity_index: EntityId,
        other_archetype: &mut Archetype,
        other_index: usize,
    ) {
        self.components[component_index].data.migrate(
            entity_index,
            &mut *other_archetype.components[other_index].data,
        );
    }

    fn len(&mut self) -> usize {
        self.entities.len()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct EntityLocation {
    archetype_index: EntityId,
    index_in_archetype: EntityId,
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
pub(crate) fn calculate_pack_id(types: &[TypeId]) -> PackId {
    let mut s = <DefaultHasher as std::default::Default>::default();
    types.hash(&mut s);
    s.finish()
}
#[derive(Clone, Copy)]
pub(crate) struct EntityMeta {
    pub(crate) generation: Generation,
    pub(crate) location: EntityLocation,
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

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub struct Entity {
    pub(crate) index: EntityId,
    pub(crate) generation: EntityId,
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
pub trait ComponentPack: 'static + Send + Sync {
    fn new_archetype(&self) -> Archetype;
    fn spawn(self, world: &mut World, entity_index: EntityId) -> EntityLocation;
}

macro_rules! component_pack {
    ($count: expr, $(($name: ident, $index: tt)),*) => {
        impl< $($name: 'static + Send + Sync),*> ComponentPack for ($($name,)*) {
            fn spawn(self, world: &mut World, entity_index: EntityId) -> EntityLocation {
                let mut types = [$(($index, TypeId::of::<$name>())), *];
                types.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                debug_assert!( types.windows(2).all(|x| x[0].1 != x[1].1), "`ComponentPack`s cannot contain duplicate components." );

                let mut order = [0; $count];
                (0..order.len()).for_each(|i|{ order[types[i].0] = i; });
                let types = [$(types[$index].1), *];

                let bundle_id = calculate_pack_id(&types);


                let archetype_index = if let Some(archetype) = world.pack_id_to_archetype.get(&bundle_id) {
                    *archetype
                } else {
                    let archetype = self.new_archetype();
                    let index = world.archetypes.len();

                    world.pack_id_to_archetype.insert(bundle_id, index);
                    world.archetypes.push(archetype);
                    index
                };

                world.archetypes[archetype_index].entities.push(entity_index);
                $(world.archetypes[archetype_index].push(order[$index], self.$index);)*
                EntityLocation {
                    archetype_index: archetype_index as EntityId,
                    index_in_archetype: (world.archetypes[archetype_index].len() - 1) as EntityId
                }
            }

            fn new_archetype(&self) -> Archetype {
                let mut components = vec![$(ComponentStore::new::<$name>()), *];
                components.sort_unstable_by(|a, b| a.type_id.cmp(&b.type_id));
                Archetype { components, entities: Vec::new() }
            }
        }
    }
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
component_pack! {2, (A, 0), (B, 1)}
component_pack! {3, (A, 0), (B, 1), (C, 2)}
component_pack! {4, (A, 0), (B, 1), (C, 2), (D, 3)}
component_pack! {5, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4)}
component_pack! {6, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5)}
component_pack! {7, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6)}
component_pack! {8, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7)}
component_pack! {9, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8)}
component_pack! {10, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9)}
component_pack! {11, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10)}
component_pack! {12, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11)}
component_pack! {13, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12)}
component_pack! {14, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13)}
component_pack! {15, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13), (O, 14)}
component_pack! {16, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13), (O, 14), (P, 15)}

#[cfg(test)]
pub mod tests {

    use glam::{Mat3, Quat, Vec3};

    use crate::physics::math::physics_system;

    use super::*;

    #[test]
    fn create_world() {
        struct Health(f32);
        struct Name(String);

        let mut world = World::new();
    }

    #[test]
    fn test_collision() {
        let mut world = World::new();
        world
            .new_entity((
                Name("A".to_string()),
                RigidBody {
                    inverse_mass: 1.0,
                    transform: Transform {
                        position: Vec3::new(0.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.0, 1.0, 1.0),
                    },
                    acceleration: Vec3::new(0.0, 0.0, 0.0),
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    angular_velocity: Vec3::new(0.0, 0.0, 0.0),
                    inverse_inertia_tensor: Mat3::IDENTITY,
                    force_accumulator: Default::default(),
                    torque_accumulator: Default::default(),
                    gravity: false,
                    restitution: 0.0,
                    is_static: true,
                    angular_drag: 0.01,
                },
                obb::DynamicOBB::new(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Quat::IDENTITY,
                ),
            ))
            .unwrap();
        world
            .new_entity((
                Name("B".to_string()),
                RigidBody {
                    inverse_mass: 1.0,
                    transform: Transform {
                        position: Vec3::new(10.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.0, 1.0, 1.0),
                    },
                    acceleration: Vec3::new(0.0, 0.0, 0.0),
                    velocity: Vec3::new(-2.0, 0.0, 0.0),
                    angular_velocity: Vec3::new(0.0, 0.0, 0.0),
                    inverse_inertia_tensor: Mat3::IDENTITY,
                    force_accumulator: Default::default(),
                    torque_accumulator: Default::default(),
                    gravity: false,
                    restitution: 0.0,
                    is_static: true,
                    angular_drag: 0.01,
                },
                obb::DynamicOBB::new(
                    Vec3::new(10.0, 0.0, 0.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Quat::IDENTITY,
                ),
            ))
            .unwrap();
        for _ in 0..100 {
            physics_print(&world);
            physics_system.run(&mut world, 0.1).unwrap();
        }
    }
    struct Name(String);

    fn physics_print(world: &World) {
        let mut search = world.search::<(&Name, &RigidBody)>().unwrap();
        for (name, rb) in search.iter() {
            println!("{} {:?} {:?}", name.0, rb.transform.position, rb.velocity);
        }
    }
}
