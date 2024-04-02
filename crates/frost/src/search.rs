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
    pub(crate) data: <T as SearchParameterGet<'world>>::RetrieveItem,
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

pub trait SearchParameterGet<'nw> {
    type RetrieveItem;

    fn retrieve(world: &'nw World, archetype: usize) -> Result<Self::RetrieveItem, RetrieveError>;
}

#[doc(hidden)]
pub struct ReadSearchParameterGet<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> SearchParameterGet<'a> for ReadSearchParameterGet<T> {
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
    type SearchParameterGet = Self;

    fn matches_archetype(_: &Archetype) -> bool {
        true
    }
}

#[doc(hidden)]
pub struct WriteSearchParameterGet<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'world, T: 'static> SearchParameterGet<'world> for WriteSearchParameterGet<T> {
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
    <<S as SearchParameter>::SearchParameterGet as SearchParameterGet<'world>>::RetrieveItem;

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
                        <<A as SearchParameter>::SearchParameterGet as SearchParameterGet<
                            'world,
                        >>::RetrieveItem,
                        <<B as SearchParameter>::SearchParameterGet as SearchParameterGet<
                            'world,
                        >>::RetrieveItem,
                    )| a.iter().zip(b.iter()),
                )
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
            type RetrieveItem = Vec<($(<$name::SearchParameterGet as SearchParameterGet<'world>>::RetrieveItem),*)>;

            fn retrieve(world: &'world World, _: usize) -> Result<Self::RetrieveItem, RetrieveError> {
                let mut archetype_indices = Vec::new();
                for (i, archetype) in world.archetypes.iter().enumerate() {
                    let matches = $($name::matches_archetype(&archetype))&&*;
                    if matches {
                        archetype_indices.push(i);
                    }
                }

                let mut result = Vec::with_capacity(archetype_indices.len());
                for index in archetype_indices {
                    result.push(($(<$name::SearchParameterGet as SearchParameterGet<'world>>::retrieve(world, index)?),*));
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

    ($x: ident, $($y: ident),*) => {
        search_params!{$x, $($y),*}
        search_paramsr!{$($y),*}
    };
}

search_paramsr!{ A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z}

search_iter! {Zip3, A, B, C}
search_iter! {Zip4, A, B, C, D}
search_iter! {Zip5, A, B, C, D, E}
search_iter! {Zip6, A, B, C, D, E, F}
search_iter! {Zip7, A, B, C, D, E, F, G}
search_iter! {Zip8, A, B, C, D, E, F, G, H}
search_iter! {Zip9, A, B, C, D, E, F, G, H, I}
search_iter! {Zip10, A, B, C, D, E, F, G, H, I, J}
search_iter! {Zip11, A, B, C, D, E, F, G, H, I, J, K}
search_iter! {Zip12, A, B, C, D, E, F, G, H, I, J, K, L}
search_iter! {Zip13, A, B, C, D, E, F, G, H, I, J, K, L, M}
search_iter! {Zip14, A, B, C, D, E, F, G, H, I, J, K, L, M, N}
search_iter! {Zip15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O}
search_iter! {Zip16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P}
search_iter! {Zip17, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q}
