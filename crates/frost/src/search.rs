use crate::iter::*;
use crate::{
    Archetype, ChainedIterator, ComponentAlreadyBorrowed, ComponentDoesNotExist, FetchError, World,
};
use std::iter::Zip;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::{any::TypeId, usize};

pub trait SystemParameter {
    // This is used to specify how and what to request from the World.
    type Fetch: for<'a> Fetch<'a>;
}

impl<'a, T: SearchParameters> SystemParameter for Search<'a, T> {
    type Fetch = SearchFetch<T>;
}

impl<T: 'static> SystemParameter for &T {
    type Fetch = Self;
}

impl<T: 'static> SystemParameter for &mut T {
    type Fetch = Self;
}

pub struct SearchFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world_borrow, T: SearchParameters> Fetch<'world_borrow> for SearchFetch<T> {
    type Item = Option<Search<'world_borrow, T>>;
    fn fetch(world: &'world_borrow World) -> Result<Self::Item, FetchError> {
        Ok(Some(Search {
            data: T::fetch(world, 0)?,
            _world: world,
        }))
    }
}

pub trait FetchItem<'a> {
    type InnerItem;
    fn inner(&'a mut self) -> Self::InnerItem;
}

pub trait Fetch<'world_borrow> {
    type Item: for<'a> FetchItem<'a>;
    fn fetch(world: &'world_borrow World) -> Result<Self::Item, FetchError>;
}

pub struct Search<'world_borrow, T: SearchParameters> {
    data: <T as SearchParameterFetch<'world_borrow>>::FetchItem,
    _world: &'world_borrow World,
}

impl<'a, 'world_borrow, T: SearchParameters> FetchItem<'a> for Option<Search<'world_borrow, T>> {
    type InnerItem = Search<'world_borrow, T>;
    fn inner(&'a mut self) -> Self::InnerItem {
        self.take().unwrap()
    }
}

impl<'a, 'world_borrow, T: 'a> FetchItem<'a> for RwLockReadGuard<'world_borrow, T> {
    type InnerItem = &'a T;
    fn inner(&'a mut self) -> Self::InnerItem {
        self
    }
}

impl<'a, 'world_borrow, T: 'a> FetchItem<'a> for RwLockWriteGuard<'world_borrow, T> {
    type InnerItem = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &mut *self
    }
}

pub struct Single<'world_borrow, T> {
    borrow: RwLockReadGuard<'world_borrow, Vec<T>>,
}

impl<'a, 'world_borrow, T: 'a> FetchItem<'a> for Single<'world_borrow, T> {
    type InnerItem = &'a T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &self.borrow[0]
    }
}

pub struct SingleMut<'world_borrow, T> {
    borrow: RwLockWriteGuard<'world_borrow, Vec<T>>,
}

impl<'a, 'world_borrow, T: 'a> FetchItem<'a> for SingleMut<'world_borrow, T> {
    type InnerItem = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerItem {
        &mut self.borrow[0]
    }
}

impl<'world_borrow, T: 'static> Fetch<'world_borrow> for &T {
    type Item = Single<'world_borrow, T>;
    fn fetch(world: &'world_borrow World) -> Result<Self::Item, FetchError> {
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id.eq(&type_id) {
                    let borrow = archetype.get(i).try_read().unwrap();
                    return Ok(Single { borrow });
                }
            }
        }

        Err(FetchError::ComponentDoesNotExist(Default::default()))
    }
}

impl<'world_borrow, T: 'static> Fetch<'world_borrow> for &mut T {
    type Item = SingleMut<'world_borrow, T>;
    fn fetch(world: &'world_borrow World) -> Result<Self::Item, FetchError> {
        // The archetypes must be found here.
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id == type_id {
                    let borrow = archetype.get(i).try_write().unwrap();
                    return Ok(SingleMut { borrow });
                }
            }
        }

        Err(FetchError::ComponentDoesNotExist(
            Default::default(),
        ))
    }
}

// Request the data from the world for a specific lifetime.
// This could instead be part of SearchParameter if Generic Associated Types were done.
pub trait SearchParameterFetch<'a> {
    type FetchItem;
    fn fetch(world: &'a World, archetype: usize) -> Result<Self::FetchItem, FetchError>;
}

#[doc(hidden)]
pub struct ReadSearchParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> SearchParameterFetch<'a> for ReadSearchParameterFetch<T> {
    type FetchItem = RwLockReadGuard<'a, Vec<T>>;
    fn fetch(world: &'a World, archetype: usize) -> Result<Self::FetchItem, FetchError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let index = archetype
            .components
            .iter()
            .position(|c| c.type_id == type_id)
            .unwrap();
        if let Ok(read_guard) = archetype.get(index).try_read() {
            Ok(read_guard)
        } else {
            Err(FetchError::ComponentAlreadyBorrowed(
                Default::default(),
            ))
        }
    }
}

// SearchParameter should fetch its own data, but the data must be requested for any lifetime
// so an inner trait must be used instead.
// 'SearchParameter' specifies the nature of the data requested, but not the lifetime.
// In the future this can (hopefully) be made better with Generic Associated Types.
pub trait SearchParameter {
    type SearchParameterFetch: for<'a> SearchParameterFetch<'a>;
    fn matches_archetype(archetype: &Archetype) -> bool;
}

impl<T: 'static> SearchParameter for &T {
    type SearchParameterFetch = ReadSearchParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

impl<T: 'static> SearchParameter for &mut T {
    type SearchParameterFetch = WriteSearchParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

pub struct Has<T> {
    pub value: bool,
    phantom: std::marker::PhantomData<T>,
}

impl<'world_borrow, T: 'static> SearchParameterFetch<'world_borrow> for Has<T> {
    type FetchItem = bool;
    fn fetch(world: &'world_borrow World, archetype: usize) -> Result<Self::FetchItem, FetchError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let contains = archetype.components.iter().any(|c| c.type_id == type_id);
        Ok(contains)
    }
}

// If a boolean value is reported, just repeat its result.
impl<'a, 'world_borrow> SearchIter<'a> for bool {
    type Iter = std::iter::Repeat<bool>;
    fn iter(&'a mut self) -> Self::Iter {
        std::iter::repeat(*self)
    }
}

impl<T: 'static> SearchParameter for Has<T> {
    type SearchParameterFetch = Self;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

#[doc(hidden)]
pub struct WriteSearchParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world_borrow, T: 'static> SearchParameterFetch<'world_borrow> for WriteSearchParameterFetch<T> {
    type FetchItem = RwLockWriteGuard<'world_borrow, Vec<T>>;
    fn fetch(world: &'world_borrow World, archetype: usize) -> Result<Self::FetchItem, FetchError> {
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
            Err(FetchError::ComponentAlreadyBorrowed(
                ComponentAlreadyBorrowed::new::<T>(),
            ))
        }
    }
}

pub trait SearchParameters: for<'a> SearchParameterFetch<'a> {}

macro_rules! Search_parameters_impl {
    ($($name: ident),*) => {
        impl<'world_borrow, $($name: SearchParameter,)*> SearchParameters
            for ($($name,)*)
        {}

        impl<'world_borrow, $($name: SearchParameter,)*> SearchParameterFetch<'world_borrow> for ($($name,)*) {
            #[allow(unused_parens)]
            type FetchItem = Vec<($(<$name::SearchParameterFetch as SearchParameterFetch<'world_borrow>>::FetchItem),*)>;

            fn fetch(world: &'world_borrow World, _archetype: usize) -> Result<Self::FetchItem, FetchError> {
                let mut archetype_indices = Vec::new();
                for (i, archetype) in world.archetypes.iter().enumerate() {
                    let matches = $($name::matches_archetype(&archetype))&&*;
                    if matches {
                        archetype_indices.push(i);
                    }
                }

                let mut result = Vec::with_capacity(archetype_indices.len());
                for index in archetype_indices {
                    result.push(($(<$name::SearchParameterFetch as SearchParameterFetch<'world_borrow>>::fetch(world, index)?),*));
                }

                Ok(result)
            }

        }
    };
}

Search_parameters_impl! {A}
Search_parameters_impl! {A, B}
Search_parameters_impl! {A, B, C}
Search_parameters_impl! {A, B, C, D}
Search_parameters_impl! {A, B, C, D, E}
Search_parameters_impl! {A, B, C, D, E, F}
Search_parameters_impl! {A, B, C, D, E, F, G}
Search_parameters_impl! {A, B, C, D, E, F, G, H}
Search_parameters_impl! {A, B, C, D, E, F, G, H, I}
Search_parameters_impl! {A, B, C, D, E, F, G, H, I, J, K}
Search_parameters_impl! {A, B, C, D, E, F, G, H, I, J, K, L}

type SearchParameterItem<'world_borrow, Q> =
    <<Q as SearchParameter>::SearchParameterFetch as SearchParameterFetch<'world_borrow>>::FetchItem;

pub trait SearchIter<'a> {
    type Iter: Iterator;
    fn iter(&'a mut self) -> Self::Iter;
}

impl<'a, 'world_borrow, T: 'static> SearchIter<'a> for RwLockReadGuard<'world_borrow, Vec<T>> {
    type Iter = std::slice::Iter<'a, T>;
    fn iter(&'a mut self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'world_borrow, T: 'static> SearchIter<'a> for RwLockWriteGuard<'world_borrow, Vec<T>> {
    type Iter = std::slice::IterMut<'a, T>;
    fn iter(&'a mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

impl<'a, 'world_borrow, A: SearchParameter> SearchIter<'a> for Search<'world_borrow, (A,)>
where
    SearchParameterItem<'world_borrow, A>: SearchIter<'a>,
{
    type Iter = ChainedIterator<SearchParameterIter<'a, 'world_borrow, A>>;
    fn iter(&'a mut self) -> Self::Iter {
        ChainedIterator::new(self.data.iter_mut().map(|v| v.iter()).collect())
    }
}

type SearchParameterIter<'a, 'world_borrow, A> =
    <SearchParameterItem<'world_borrow, A> as SearchIter<'a>>::Iter;
impl<'a, 'world_borrow, A: SearchParameter, B: SearchParameter> SearchIter<'a>
    for Search<'world_borrow, (A, B)>
where
    SearchParameterItem<'world_borrow, A>: SearchIter<'a>,
    SearchParameterItem<'world_borrow, B>: SearchIter<'a>,
{
    type Iter = ChainedIterator<
        Zip<SearchParameterIter<'a, 'world_borrow, A>, SearchParameterIter<'a, 'world_borrow, B>>,
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
        impl<'a, 'world_borrow, $($name: SearchParameter),*> SearchIter<'a> for Search<'world_borrow, ($($name,)*)>
        where
            $(SearchParameterItem<'world_borrow, $name>: SearchIter<'a>),*
             {
            type Iter = ChainedIterator<$zip_type<$(SearchParameterIter<'a, 'world_borrow, $name>,)*>>;
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

search_iter! {Zip3, A, B, C}
search_iter! {Zip4, A, B, C, D}
search_iter! {Zip5, A, B, C, D, E}
search_iter! {Zip6, A, B, C, D, E, F}
search_iter! {Zip7, A, B, C, D, E, F, G}
search_iter! {Zip8, A, B, C, D, E, F, G, H}