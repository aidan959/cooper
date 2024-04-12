pub mod bounding_box;
pub mod entity;
pub mod obb;
pub mod shapes;

mod mass;
mod math;

mod component_utils;
mod input;
pub mod physics;
use component_utils::{calculate_pack_id, component_vec_to_mut};
mod iter;
mod utils;
use utils::get_two_mutable;
mod errors;
mod search;
mod system;
use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    collections::{hash_map::DefaultHasher, HashMap},
    sync::{Mutex, RwLock},
};

use crate::{Get, GetError, SearchGet, SearchParameters, Single, SingleMut};
pub use input::Input;
pub(crate) type EntityId = u32;
pub(crate) type Generation = u32;

pub use iter::*;
pub use physics::*;
pub use search::Search;
pub use search::*;

pub use errors::*;
pub use system::*;

pub trait ComponentPack: 'static + Send + Sync {
    fn new_archetype(&self) -> Archetype;
    fn spawn_in_world(self, world: &mut World, entity_index: EntityId) -> EntityLocation;
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
    pub(crate) fn get<T: 'static>(&self, index: usize) -> &RwLock<Vec<T>> {
        self.components[index]
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>()
            .unwrap()
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
        component_vec_to_mut(&mut *self.components[component_index].data)
    }

    fn replace_component<T: 'static>(&mut self, component_index: usize, index: EntityId, t: T) {
        self.mutable_component_store(component_index)[index as usize] = t;
    }

    fn push<T: 'static>(&mut self, component_index: usize, t: T) {
        self.mutable_component_store(component_index).push(t)
    }

    fn get_component_mut<T: 'static>(
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
        component_vec_to_mut(other_component_vec).push(data);
    }

    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync> {
        Box::new(RwLock::new(Vec::<T>::new()))
    }
}
pub struct World {
    archetypes: Vec<Archetype>,
    pack_id_to_archetype: HashMap<u64, usize>,
    pub(crate) entities: Vec<EntityMeta>,
    available_entities: Vec<EntityId>,
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
    pub fn new_entity(&mut self, b: impl ComponentPack) -> Result<Entity, WorldFull> {
        let (index, generation) = if let Some(index) = self.available_entities.pop() {
            let (generation, _) = self.entities[index as usize].generation.overflowing_add(1);
            (index, generation)
        } else {
            self.entities.push(EntityMeta::null());

            if self.entities.len() >= EntityId::MAX as usize {
                return Err(WorldFull::new());
            }
            ((self.entities.len() - 1) as EntityId, 0)
        };

        self.entities[index as usize] = EntityMeta {
            location: b.spawn_in_world(self, index),
            generation: generation,
        };
        Ok(Entity { index, generation })
    }
    // gets a single immutable reference
    pub fn get_single<T: 'static>(&self) -> Result<Single<T>, GetError> {
        <&T>::get(self)
    }
    // gets a single mutable reference
    pub fn get_single_mut<T: 'static>(&self) -> Result<SingleMut<T>, GetError> {
        <&mut T>::get(self)
    }

    /**
    Search from the world.
    # EX
    ```
    use frost::*;
    let mut world = World::new();
    let search = world.search<(&bool, &String)>();
    ```
    */
    pub fn search<'world_borrow, T: SearchParameters>(
        &'world_borrow self,
    ) -> Result<Search<T>, GetError> {
        let get = SearchGet::<T>::get(self);

        match get {
            Ok(mut search_get) => Ok(search_get.take().unwrap()),
            Err(e) => Err(e),
        }
    }
    pub fn add_component<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
        t: T,
    ) -> Result<(), EntityNotFound> {
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

        match type_index {
            Ok(insert_index) => {
                let archetype = &mut self.archetypes[entity_meta.archetype_index() as usize];

                archetype.replace_component(insert_index, entity_meta.index_in_archetype(), t);
            }
            Err(_) => {
                let insert_index = type_index.unwrap_or_else(|err| err);
                type_ids.insert(insert_index, type_id);
                let pack_id = calculate_pack_id(&type_ids);

                let new_archetype_index = match self.pack_id_to_archetype.get(&pack_id) {
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

                let (old_archetype, new_archetype) = get_two_mutable(
                    &mut self.archetypes,
                    entity_meta.archetype_index() as usize,
                    new_archetype_index,
                );
                if let Some(last) = old_archetype.entities.last() {
                    self.entities[*last as usize].location = entity_meta.location;
                }
                self.entities[entity.index as usize].location = EntityLocation::new(
                    new_archetype_index as EntityId,
                    new_archetype.len() as EntityId,
                );

                (0..insert_index).into_iter().for_each(|i| {
                    old_archetype.migrate_component(
                        i,
                        entity_meta.index_in_archetype(),
                        new_archetype,
                        i,
                    )
                });

                new_archetype.push(insert_index, t);

                let components_in_archetype = old_archetype.components.len();

                (insert_index..components_in_archetype)
                    .into_iter()
                    .for_each(|i| {
                        old_archetype.migrate_component(
                            i,
                            entity_meta.index_in_archetype(),
                            new_archetype,
                            i.overflowing_add(1).0,
                        )
                    });

                old_archetype
                    .entities
                    .swap_remove(entity_meta.index_in_archetype() as usize);
                new_archetype.entities.push(entity.index);
            }
        }

        Ok(())
    }
}
macro_rules! component_pack_impl {
    ($count: expr, $(($name: ident, $index: tt)),*) => {
        impl< $($name: 'static + Send + Sync),*> ComponentPack for ($($name,)*) {
            fn new_archetype(&self) -> Archetype {
                let mut components = vec![$(ComponentStore::new::<$name>()), *];
                components.sort_unstable_by(|a, b| a.type_id.cmp(&b.type_id));
                Archetype { components, entities: Vec::new() }
            }

            fn spawn_in_world(self, world: &mut World, entity_index: EntityId) -> EntityLocation {
                let mut types = [$(($index, TypeId::of::<$name>())), *];
                types.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                debug_assert!(
                    types.windows(2).all(|x| x[0].1 != x[1].1),
                    "`ComponentPack`s can't contain duplicate components."
                );

                let mut order = [0; $count];
                for i in 0..order.len() {
                    order[types[i].0] = i;
                }
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
        }
    }
}

component_pack_impl! {1, (A, 0)}
component_pack_impl! {2, (A, 0), (B, 1)}
component_pack_impl! {3, (A, 0), (B, 1), (C, 2)}
component_pack_impl! {4, (A, 0), (B, 1), (C, 2), (D, 3)}
component_pack_impl! {5, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4)}
component_pack_impl! {6, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5)}

#[cfg(test)]
pub mod tests {
    use std::{cell::RefCell, ops::DerefMut};

    use glam::{Mat3, Quat, Vec3};

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
                    gravity: false,
                    restitution: 0.0,
                },
                obb::OBB::new(
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
                    gravity: false,
                    restitution: 0.0,
                },
                obb::OBB::new(
                    Vec3::new(10.0, 0.0, 0.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Quat::IDENTITY,
                ),
            ))
            .unwrap();
        for _ in 0..100 {
            physics_print(&world);
            physics_system.run(&mut world, 0.1).unwrap();
            println!("");
        }
    }
    struct Name(String);

    fn physics_print(world: &World) {
        let mut search = world.search::<(&Name, &RigidBody)>().unwrap();
        for (name, rb) in search.iter() {
            println!("{} {:?} {:?}", name.0, rb.transform.position, rb.velocity);
        }
    }
    fn physics_system(mut search: Search<(&mut RigidBody, &mut obb::OBB)>, fixed_update: f32) {
        let bodies_and_boxes = search.iter().collect::<Vec<_>>();

        let mut collision_details = Vec::new();
        for i in 0..bodies_and_boxes.len() {
            for j in (i + 1)..bodies_and_boxes.len() {
                let (_, obb1) = &bodies_and_boxes[i];
                let (_, obb2) = &bodies_and_boxes[j];

                if let Some(collision_point) = obb1.get_collision_point_normal(obb2) {
                    collision_details.push((i, j, collision_point));
                }
            }
        }
        let mut bodies_and_boxes: Vec<RefCell<(&mut RigidBody, &mut obb::OBB)>> = search
            .iter()
            .map(|(rb, obb)| RefCell::new((rb, obb)))
            .collect::<Vec<RefCell<(&mut RigidBody, &mut obb::OBB)>>>();
        for collision in collision_details {
            let rb1 = &mut bodies_and_boxes[collision.0].borrow_mut().0;
            let rb2 = &mut bodies_and_boxes[collision.1].borrow_mut().0;
            handle_collision(rb1, rb2, &collision.2);
        }

        // Update positions after handling all collisions
        bodies_and_boxes.iter_mut().for_each(|b_b| {
            let (rb, obb) = b_b.borrow_mut().get_mut();

            rb.acceleration += Vec3::new(0., -9.81, 0.);
            integrate_velocity(&mut rb.velocity, rb.acceleration, fixed_update);
            integrate_position(&mut rb.transform, rb.velocity, fixed_update);
            obb.center = rb.transform.position;
            obb.orientation = rb.transform.rotation;
        });
    }


    
}
