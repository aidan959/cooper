use crate::{ComponentStore, Archetype, World};
use std::any::TypeId;
use std::iter::Zip;
use std::slice::{Iter, IterMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait SysParam {
    type Retrieve: for<'a> Retrieve<'a>;
}

impl<'a, T: SearchParameters> SysParam for Search<'a, T> {
    type Retrieve = SearchRetrieve<T>;
}

impl<T: 'static> SysParam for &T {
    type Retrieve = Self;
}

impl<T: 'static> SysParam for &mut T {
    type Retrieve = Self;
}


pub struct SearchRetrieve<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: SearchParameters> Retrieve<'world> for SearchRetrieve<T> {
    type Item = Option<Search<'world, T>>;
    fn retrieve(world: &'world World) -> Result<Self::Item, RetrieveError> {
        Ok(Some(Search {
            data: T::retrieve(world, 0)?
        }))
    }
}
macro_rules! Search_impl {
    ($count: expr, $(($name: ident, $index: tt)),*) => {
        impl<'a, 'b: 'a, $($name: ComponentSearchTrait<'a, 'b>),*> Search<'a, 'b> for ($($name,)*) {
            type ITERATOR = SearchIterator<($($name::ITERATOR,)*)>;

            fn iterator(&'b mut self) -> Self::ITERATOR {
               SearchIterator(($(self.$index.iterator(),)*))
            }
        }
    };
}

impl<'a, 'b: 'a, A: ComponentSearchTrait<'a, 'b>> Search<'a, 'b> for (A,) {
    type ITERATOR = A::ITERATOR;
    fn iterator(&'b mut self) -> Self::ITERATOR {
        self.0.iterator()
    }
}

impl<'a, 'b: 'a, A: ComponentSearchTrait<'a, 'b>, B: ComponentSearchTrait<'a, 'b>> Search<'a, 'b>
    for (A, B)
{
    type ITERATOR = Zip<A::ITERATOR, B::ITERATOR>;
    fn iterator(&'b mut self) -> Self::ITERATOR {
        self.0.iterator().zip(self.1.iterator())
    }
}

pub trait SearchParam<'a, 'b: 'a> {
    type ComponentSearch: ComponentSearchTrait<'a, 'b>;
    fn type_id() -> TypeId;
    fn get_component_Search(archetypes: &Vec<SearchableArchetype>) -> Self::ComponentSearch;
}

impl<'a, 'b: 'a, T: 'static> SearchParam<'a, 'b> for &T {
    type ComponentSearch = ComponentSearch<'b, T>;
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn get_component_Search(archetypes: &Vec<SearchableArchetype>) -> Self::ComponentSearch {
        // Need to get immutable access to all the world components here.
        unimplemented!()
    }
}

impl<'a, 'b: 'a, T: 'static> SearchParam<'a, 'b> for &mut T {
    type ComponentSearch = MutableComponentSearch<'b, T>;
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn get_component_Search(archetypes: &Vec<SearchableArchetype>) -> Self::ComponentSearch {
        // Need to get mutable access to all the world components here.
        unimplemented!()
    }
}

pub trait ComponentSearchTrait<'a, 'b: 'a> {
    type ITERATOR: Iterator;
    fn new() -> Self;
    fn add_archetype(&'b mut self, archetype: &'b SearchableArchetype, component_index: usize);
    fn iterator(&'b mut self) -> Self::ITERATOR;
}

pub struct MutableComponentSearch<'a, T: 'a> {
    guards: Vec<RwLockWriteGuard<'a, Box<dyn ComponentStore>>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, 'b: 'a, T: 'static> ComponentSearchTrait<'a, 'b> for MutableComponentSearch<'b, T> {
    type ITERATOR = ComponentIterMut<'a, T>;

    fn new() -> Self {
        Self {
            guards: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn add_archetype(&'b mut self, archetype: &'b SearchableArchetype, component_index: usize) {
        let lock_write_guard = archetype.components[component_index].write().unwrap();
        self.guards.push(lock_write_guard);
    }

    fn iterator(&'b mut self) -> Self::ITERATOR {
        let iters = self
            .guards
            .iter_mut()
            .map(|g| g.to_any_mut().downcast_mut::<Vec<T>>().unwrap().iter_mut())
            .collect();
        ComponentIterMut::new(iters)
    }
}

pub struct ComponentSearch<'a, T: 'a> {
    guards: Vec<RwLockReadGuard<'a, Box<dyn ComponentStore>>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, 'b: 'a, T: 'static> ComponentSearchTrait<'a, 'b> for ComponentSearch<'b, T> {
    type ITERATOR = ComponentIter<'a, T>;
    fn new() -> Self {
        Self {
            guards: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn add_archetype(&'b mut self, archetype: &'b SearchableArchetype, component_index: usize) {
        let lock_read_guard = archetype.components[component_index].read().unwrap();
        self.guards.push(lock_read_guard);
    }
    fn iterator(&'b mut self) -> Self::ITERATOR {
        let iters = self
            .guards
            .iter()
            .map(|g| g.to_any().downcast_ref::<Vec<T>>().unwrap().iter())
            .collect();
        ComponentIter::new(iters)
    }
}

pub struct ComponentIter<'a, T> {
    current_iter: Iter<'a, T>,
    iterators: Vec<Iter<'a, T>>,
}

impl<'a, T> ComponentIter<'a, T> {
    pub fn new(mut iterators: Vec<Iter<'a, T>>) -> Self {
        let current_iter = iterators.pop().unwrap();
        Self {
            current_iter,
            iterators,
        }
    }
}

impl<'a, T: 'a> Iterator for ComponentIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Chain the iterators together.
        // If the end of one iterator is reached go to the next.
        self.current_iter.next().or_else(|| {
            self.iterators.pop().map_or(None, |i| {
                self.current_iter = i;
                self.current_iter.next()
            })
        })
    }
}

pub struct ComponentIterMut<'a, T> {
    current_iter: IterMut<'a, T>,
    iterators: Vec<IterMut<'a, T>>,
}

impl<'a, T> ComponentIterMut<'a, T> {
    pub fn new(mut iterators: Vec<IterMut<'a, T>>) -> Self {
        let current_iter = iterators.pop().unwrap();
        Self {
            current_iter,
            iterators,
        }
    }
}

impl<'a, T: 'a> Iterator for ComponentIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_iter.next().or_else(|| {
            self.iterators.pop().map_or(None, |i| {
                self.current_iter = i;
                self.current_iter.next()
            })
        })
    }
}
