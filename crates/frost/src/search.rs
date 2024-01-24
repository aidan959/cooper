use crate::{Archetype, World};

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

pub trait SearchParameter {
    type SearchParameterRetrieve: for<'a> SearchParameterRetrieve<'a>;
    fn matches(archetype: &Archetype) -> bool;
}


pub struct Has<T> {
    pub value: bool,
    phantom: std::marker::PhantomData<T>,
}