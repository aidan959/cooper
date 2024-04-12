use crate::{iter::*, EntityId, Generation};
use crate::{
    Archetype, ChainedIterator, ComponentAlreadyBorrowed, ComponentDoesNotExist, GetError, World,
};
use std::iter::Zip;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::{any::TypeId, usize};

pub trait SystemParameter {
    type Get: for<'a> Get<'a>;
}

impl<'a, T: SearchParameters> SystemParameter for Search<'a, T> {
    type Get = SearchGet<T>;
}

impl<T: 'static> SystemParameter for &T {
    type Get = Self;
}

impl<T: 'static> SystemParameter for &mut T {
    type Get = Self;
}

pub struct SearchGet<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: SearchParameters> Get<'world> for SearchGet<T> {
    type Item = Option<Search<'world, T>>;
    fn get(world: &'world World) -> Result<Self::Item, GetError> {
        
        Ok(Some(Search {
            data: T::get(world, 0)?,
            _world: world,
        }))
    }
}

pub trait GetItem<'a> {
    type InnerItem;
    fn inner(&'a mut self) -> Self::InnerItem;
}

pub trait Get<'world> {
    type Item: for<'a> GetItem<'a>;
    fn get(world: &'world World) -> Result<Self::Item, GetError>;
}

pub struct Search<'world, T: SearchParameters> {
    data: <T as SearchParameterGet<'world>>::GetItem,
    _world: &'world World,
}

impl<'a, 'world, T: SearchParameters> GetItem<'a> for Option<Search<'world, T>> {
    type InnerItem = Search<'world, T>;
    fn inner(&'a mut self) -> Self::InnerItem {
        self.take().unwrap()
    }
}

impl<'a, 'world, T: 'a> GetItem<'a> for RwLockReadGuard<'world, T> {
    type InnerItem = &'a T;
    fn inner(&'a mut self) -> Self::InnerItem {
        self
    }
}

impl<'a, 'world, T: 'a> GetItem<'a> for RwLockWriteGuard<'world, T> {
    type InnerItem = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &mut *self
    }
}

pub struct Single<'world, T> {
    borrow: RwLockReadGuard<'world, Vec<T>>,
}

impl<'a, 'world, T: 'a> GetItem<'a> for Single<'world, T> {
    type InnerItem = &'a T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &self.borrow[0]
    }
}

pub struct SingleMut<'world, T> {
    borrow: RwLockWriteGuard<'world, Vec<T>>,
}

impl<'a, 'world, T: 'a> GetItem<'a> for SingleMut<'world, T> {
    type InnerItem = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &mut self.borrow[0]
    }
}

impl<'world, T: 'static> Get<'world> for &T {
    type Item = Single<'world, T>;
    fn get(world: &'world World) -> Result<Self::Item, GetError> {
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id.eq(&type_id) {
                    let borrow = archetype.get(i).try_read().unwrap();
                    return Ok(Single { borrow });
                }
            }
        }

        Err(GetError::ComponentDoesNotExist(Default::default()))
    }
}

impl<'world, T: 'static> Get<'world> for &mut T {
    type Item = SingleMut<'world, T>;
    fn get(world: &'world World) -> Result<Self::Item, GetError> {
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id == type_id {
                    let borrow = archetype.get(i).try_write().unwrap();
                    return Ok(SingleMut { borrow });
                }
            }
        }

        Err(GetError::ComponentDoesNotExist(
            Default::default(),
        ))
    }
}

pub trait SearchParameterGet<'nw> {
    type GetItem;
    
    fn get(world: &'nw World, archetype: usize) -> Result<Self::GetItem, GetError>;
}

#[doc(hidden)]
pub struct ReadSearchParameterGet<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> SearchParameterGet<'a> for ReadSearchParameterGet<T> {
    type GetItem = RwLockReadGuard<'a, Vec<T>>;
    fn get(world: &'a World, archetype: usize) -> Result<Self::GetItem, GetError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let index = archetype
            .components
            .iter()
            .position(|c| c.type_id == type_id)
            .unwrap();
        if let Ok(read_guard) = archetype.get(index).try_read() {
            let id =  index;
            Ok(read_guard)
        } else {
            Err(GetError::ComponentAlreadyBorrowed(
                Default::default(),
            ))
        }
    }
}

// SearchParameter should get its own data, but the data must be requested for any lifetime
// so an inner trait must be used instead.
// 'SearchParameter' specifies the nature of the data requested, but not the lifetime.
// In the future this can (hopefully) be made better with Generic Associated Types.
pub trait SearchParameter {
    type SearchParameterGet: for<'a> SearchParameterGet<'a>;
    fn matches_archetype(archetype: &Archetype) -> bool;
}

impl<T: 'static> SearchParameter for &T {
    type SearchParameterGet = ReadSearchParameterGet<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

impl<T: 'static> SearchParameter for &mut T {
    type SearchParameterGet = WriteSearchParameterGet<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

pub struct Has<T> {
    pub value: bool,
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: 'static> SearchParameterGet<'world> for Has<T> {
    type GetItem = bool;
    fn get(world: &'world World, archetype: usize) -> Result<Self::GetItem, GetError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let contains = archetype.components.iter().any(|c| c.type_id == type_id);
        Ok(contains)
    }
}

impl<'a, 'world> SearchIter<'a> for bool {
    type Iter = std::iter::Repeat<bool>;
    fn iter(&'a mut self) -> Self::Iter {
        std::iter::repeat(*self)
    }
}

impl<T: 'static> SearchParameter for Has<T> {
    type SearchParameterGet = Self;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

#[doc(hidden)]
pub struct WriteSearchParameterGet<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: 'static> SearchParameterGet<'world> for WriteSearchParameterGet<T> {
    type GetItem = RwLockWriteGuard<'world, Vec<T>>;
    fn get(world: &'world World, archetype: usize) -> Result<Self::GetItem, GetError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let index = archetype
            .components
            .iter()
            .position(|c| c.type_id == type_id)
            .unwrap();
        if let Ok(write_guard) = archetype.get(index).try_write() {
            Ok(write_guard)
        } else {
            Err(GetError::ComponentAlreadyBorrowed(
                ComponentAlreadyBorrowed::new::<T>(),
            ))
        }
    }
}


type SearchParameterItem<'world, S> =
    <<S as SearchParameter>::SearchParameterGet as SearchParameterGet<'world>>::GetItem;

pub trait SearchIter<'a> {
    type Iter: Iterator;
    fn iter(&'a mut self) -> Self::Iter;
}

impl<'a, 'world, T: 'static> SearchIter<'a> for RwLockReadGuard<'world, Vec<T>> {
    type Iter = std::slice::Iter<'a, T>;
    fn iter(&'a mut self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'world, T: 'static> SearchIter<'a> for RwLockWriteGuard<'world, Vec<T>> {
    type Iter = std::slice::IterMut<'a, T>;
    fn iter(&'a mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

impl<'a, 'world, A: SearchParameter> SearchIter<'a> for Search<'world, (A,)>
where
    SearchParameterItem<'world, A>: SearchIter<'a>,
{
    type Iter = ChainedIterator<SearchParameterIter<'a, 'world, A>>;
    fn iter(&'a mut self) -> Self::Iter {
        ChainedIterator::new(self.data.iter_mut().map(|v| v.iter()).collect())
    }
}

type SearchParameterIter<'a, 'world, A> =
    <SearchParameterItem<'world, A> as SearchIter<'a>>::Iter;
impl<'a, 'world, A: SearchParameter, B: SearchParameter> SearchIter<'a>
    for Search<'world, (A, B)>
where
    SearchParameterItem<'world, A>: SearchIter<'a>,
    SearchParameterItem<'world, B>: SearchIter<'a>,
{
    type Iter = ChainedIterator<
        Zip<SearchParameterIter<'a, 'world, A>, SearchParameterIter<'a, 'world, B>>,
    >;
    fn iter(&'a mut self) -> Self::Iter {
        ChainedIterator::new(
            self.data
                .iter_mut()
                .map(|(a, b)| a.iter().zip(b.iter()))
                .collect(),
        )
    }
}

macro_rules! search_iter {
    ($zip_type: ident, $($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, 'world, $($name: SearchParameter),*> SearchIter<'a> for Search<'world, ($($name,)*)>
        where
            $(SearchParameterItem<'world, $name>: SearchIter<'a>),*
             {
            type Iter = ChainedIterator<$zip_type<$(SearchParameterIter<'a, 'world, $name>,)*>>;
            fn iter(&'a mut self) -> Self::Iter {
                ChainedIterator::new(
                    self.data
                    .iter_mut()
                    .map(|($(ref mut $name,)*)| $zip_type::new($($name.iter(),)*))
                    .collect()
                )
            }
        }
    }
}

pub trait SearchParameters: for<'a> SearchParameterGet<'a> {}

macro_rules! search_params {
    ($($name: ident),*) => {
        impl<'world, $($name: SearchParameter,)*> SearchParameters
            for ($($name,)*)
        {}

        impl<'world, $($name: SearchParameter,)*> SearchParameterGet<'world> for ($($name,)*) {
            #[allow(unused_parens)]
            type GetItem = Vec<($(<$name::SearchParameterGet as SearchParameterGet<'world>>::GetItem),*)>;

            fn get(world: &'world World, _archetype: usize) -> Result<Self::GetItem, GetError> {
                let mut archetype_indices = Vec::new();
                for (i, archetype) in world.archetypes.iter().enumerate() {
                    let matches = $($name::matches_archetype(&archetype))&&*;
                    if matches {
                        archetype_indices.push(i);
                    }
                }

                let mut result = Vec::with_capacity(archetype_indices.len());
                for index in archetype_indices {
                    result.push(($(<$name::SearchParameterGet as SearchParameterGet<'world>>::get(world, index)?),*));
                }

                Ok(result)
            }

        }
    };
}

macro_rules! search_paramsr{
    ($x: ident) => {
        search_params!{$x}
    };

    ($x: ident, $($y: ident),+) => {
        search_params!{$x, $($y),+}
        search_paramsr!{$($y),+}

    };
}


// TODO - this could be a little cleaner
search_paramsr! {A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z}

// TODO - I am doing this alot - switch to procedural macro?
search_iter! {Zip3, A, B, C}
search_iter! {Zip4, A, B, C, D}
search_iter! {Zip5, A, B, C, D, E}
search_iter! {Zip6, A, B, C, D, E, F}
search_iter! {Zip7, A, B, C, D, E, F, G}
search_iter! {Zip8, A, B, C, D, E, F, G, H}