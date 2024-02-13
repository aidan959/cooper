use crate::iter::*;
use crate::{Archetype, ChainedIterator, ComponentAlreadyBorrowed, RetrieveError, World};
use std::iter::Zip;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::{any::TypeId, usize};

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

pub trait RetrieveItem<'a> {
    type InnerComponent;
    fn inner(&'a mut self) -> Self::InnerComponent;
}

pub trait Retrieve<'world> {
    type Item: for<'a> RetrieveItem<'a>;
    fn retrieve(world: &'world World) -> Result<Self::Item, RetrieveError>;
}

pub struct Search<'world, T: SearchParameters> {
    pub(crate) data: <T as SearchParameterRetrieve<'world>>::RetrieveItem,
}

impl<'a, 'world, T: SearchParameters> RetrieveItem<'a> for Option<Search<'world, T>> {
    type InnerComponent = Search<'world, T>;
    fn inner(&'a mut self) -> Self::InnerComponent {
        self.take().unwrap()
    }
}

impl<'a, 'world, T: 'a> RetrieveItem<'a> for RwLockReadGuard<'world, T> {
    type InnerComponent = &'a T;
    fn inner(&'a mut self) -> Self::InnerComponent {
        self
    }
}

impl<'a, 'world, T: 'a> RetrieveItem<'a> for RwLockWriteGuard<'world, T> {
    type InnerComponent = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerComponent {
        &mut *self
    }
}

pub struct Single<'world, T> {
    borrow: RwLockReadGuard<'world, Vec<T>>,
}

impl<'a, 'world, T: 'a> RetrieveItem<'a> for Single<'world, T> {
    type InnerComponent = &'a T;
    fn inner(&'a mut self) -> Self::InnerComponent {
        &self.borrow[0]
    }
}

pub struct SingleMut<'world, T> {
    borrow: RwLockWriteGuard<'world, Vec<T>>,
}

impl<'a, 'world, T: 'a> RetrieveItem<'a> for SingleMut<'world, T> {
    type InnerComponent = &'a mut T;
    fn inner(&'a mut self) -> Self::InnerComponent {
        &mut self.borrow[0]
    }
}

impl<'world, T: 'static> Retrieve<'world> for &T {
    type Item = Single<'world, T>;
    fn retrieve(world: &'world World) -> Result<Self::Item, RetrieveError> {
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id == type_id {
                    return Ok(Single {
                        borrow: archetype.retrieve(i).try_read().unwrap(),
                    });
                }
            }
        }

        Err(RetrieveError::ComponentDoesNotExist(Default::default()))
    }
}

impl<'world, T: 'static> Retrieve<'world> for &mut T {
    type Item = SingleMut<'world, T>;
    fn retrieve(world: &'world World) -> Result<Self::Item, RetrieveError> {
        let type_id = TypeId::of::<T>();
        for archetype in world.archetypes.iter() {
            for (i, c) in archetype.components.iter().enumerate() {
                if c.type_id == type_id {
                    return Ok(SingleMut {
                        borrow: archetype.retrieve(i).try_write().unwrap(),
                    });
                }
            }
        }

        Err(RetrieveError::ComponentDoesNotExist(Default::default()))
    }
}

pub trait SearchParameterRetrieve<'nw> {
    type RetrieveItem;

    fn retrieve(world: &'nw World, archetype: usize) -> Result<Self::RetrieveItem, RetrieveError>;
}

#[doc(hidden)]
pub struct ReadSearchParameterRetrieve<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> SearchParameterRetrieve<'a> for ReadSearchParameterRetrieve<T> {
    type RetrieveItem = RwLockReadGuard<'a, Vec<T>>;
    fn retrieve(world: &'a World, archetype: usize) -> Result<Self::RetrieveItem, RetrieveError> {
        let archetype: &Archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let index = archetype
            .components
            .iter()
            .position(|c| c.type_id == type_id)
            .unwrap();
        if let Ok(read_guard) = archetype.retrieve(index).try_read() {
            Ok(read_guard)
        } else {
            Err(RetrieveError::ComponentAlreadyBorrowed(Default::default()))
        }
    }
}

pub trait SearchParameter {
    type SearchParameterRetrieve: for<'a> SearchParameterRetrieve<'a>;
    fn matches(archetype: &Archetype) -> bool;
}

impl<T: 'static> SearchParameter for &T {
    type SearchParameterRetrieve = ReadSearchParameterRetrieve<T>;

    fn matches(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

impl<T: 'static> SearchParameter for &mut T {
    type SearchParameterRetrieve = WriteSearchParameterRetrieve<T>;

    fn matches(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.components.iter().any(|c| c.type_id == type_id)
    }
}

pub struct Has<T> {
    pub value: bool,
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: 'static> SearchParameterRetrieve<'world> for Has<T> {
    type RetrieveItem = bool;
    fn retrieve(
        world: &'world World,
        archetype: usize,
    ) -> Result<Self::RetrieveItem, RetrieveError> {
        let archetype = &world.archetypes[archetype];
        let type_id = TypeId::of::<T>();

        let contains = archetype.components.iter().any(|c| c.type_id == type_id);
        Ok(contains)
    }
}

impl<'a, 'world> SearchIter<'a> for bool {
    type Iter = std::iter::Repeat<bool>;
    fn iter(&'a mut self) -> Self::Iter {
        std::iter::repeat::<_>(*self)
    }
}

impl<T: 'static> SearchParameter for Has<T> {
    type SearchParameterRetrieve = Self;

    fn matches(_: &Archetype) -> bool {
        true
    }
}

#[doc(hidden)]
pub struct WriteSearchParameterRetrieve<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: 'static> SearchParameterRetrieve<'world> for WriteSearchParameterRetrieve<T> {
    type RetrieveItem = RwLockWriteGuard<'world, Vec<T>>;
    fn retrieve(
        world: &'world World,
        archetype: usize,
    ) -> Result<Self::RetrieveItem, RetrieveError> {
        let archetype = &world.archetypes[archetype];
        let type_id: TypeId = TypeId::of::<T>();

        let index: usize = archetype
            .components
            .iter()
            .position(|c| c.type_id == type_id)
            .unwrap();
        if let Ok(write_guard) = archetype.retrieve(index).try_write() {
            Ok(write_guard)
        } else {
            Err(RetrieveError::ComponentAlreadyBorrowed(
                ComponentAlreadyBorrowed::new::<T>(),
            ))
        }
    }
}

type SearchParameterItem<'world, S> =
    <<S as SearchParameter>::SearchParameterRetrieve as SearchParameterRetrieve<'world>>::RetrieveItem;

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

type SearchParameterIter<'a, 'world, A> = <SearchParameterItem<'world, A> as SearchIter<'a>>::Iter;
impl<'a, 'world, A: SearchParameter, B: SearchParameter> SearchIter<'a> for Search<'world, (A, B)>
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
                .map(
                    |(a, b): &mut (
                        <<A as SearchParameter>::SearchParameterRetrieve as SearchParameterRetrieve<
                            'world,
                        >>::RetrieveItem,
                        <<B as SearchParameter>::SearchParameterRetrieve as SearchParameterRetrieve<
                            'world,
                        >>::RetrieveItem,
                    )| a.iter().zip(b.iter()),
                )
                .collect(),
        )
    }
}


pub trait SearchParameters: for<'a> SearchParameterRetrieve<'a> {}

macro_rules! search_params {
    ($($name: ident),*) => {
        impl<'world, $($name: SearchParameter,)*> SearchParameters for ($($name,)*){}

        impl<'world, $($name: SearchParameter,)*> SearchParameterRetrieve<'world> for ($($name,)*) {
            #[allow(unused_parens)]
            type RetrieveItem = Vec<($(<$name::SearchParameterRetrieve as SearchParameterRetrieve<'world>>::RetrieveItem),*)>;

            fn retrieve(world: &'world World, _: usize) -> Result<Self::RetrieveItem, RetrieveError> {
                let mut archetype_indices = Vec::new();
                world.archetypes.iter().enumerate() .for_each(|(i, archetype)| { if $($name::matches(&archetype))&&* {archetype_indices.push(i) } });

                let mut result = Vec::with_capacity(archetype_indices.len());
                for index in archetype_indices {
                    result.push(($(<$name::SearchParameterRetrieve as SearchParameterRetrieve<'world>>::retrieve(world, index)?),*));
                }

                Ok(result)
            }

        }
    };
}


search_params!{A}A, B, C
search_params!{A}A, B, C, D
search_params!{A}A, B, C, D, E
search_params!{A}A, B, C, D, E, F
search_params!{A}A, B, C, D, E, F, G
search_params!{A}A, B, C, D, E, F, G, H
search_params!{A}A, B, C, D, E, F, G, H, I
search_params!{A}A, B, C, D, E, F, G, H, I, J
search_params!{A}A, B, C, D, E, F, G, H, I, J, K
search_params!{A}A, B, C, D, E, F, G, H, I, J, K, L
search_params!{A}A, B, C, D, E, F, G, H, I, J, K, L, M
search_params!{A}A, B, C, D, E, F, G, H, I, J, K, L, M, N

