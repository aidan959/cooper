use crate::EntityId;

pub trait FrostError: std::error::Error + std::fmt::Display {
    fn new() -> Self;
}
#[derive(Debug)]
pub struct WorldFull {}
impl FrostError for WorldFull {
    fn new() -> Self { WorldFull{} }
}

impl std::error::Error for WorldFull{}

impl std::fmt::Display for WorldFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Too many entities created in world ({}).",
            usize::MAX
        )
    }
}


#[derive(Debug)]
pub struct EntityNotFound{missing_id: EntityId}
impl EntityNotFound {
    pub fn new_with_value(id: EntityId) -> Self{
        let mut err = Self::new();
        err.missing_id = id;
        err
    }
}
impl FrostError for EntityNotFound {
    fn new() -> Self {
        Self{missing_id: 0}
    }
}
impl std::fmt::Display for EntityNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The entity {} longer exists so the operation cannot be performed",
            self.missing_id
        )
    }
}

impl std::error::Error for EntityNotFound {}

#[derive(Debug)]
pub struct ComponentNotInEntity(EntityId, &'static str);

impl ComponentNotInEntity {
    pub fn new_with_value<T>(entity_id: EntityId) -> Self {
        Self(entity_id, std::any::type_name::<T>())
    }
}
impl std::fmt::Display for ComponentNotInEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity {:?} does not have a related [{}] component",
            self.0, self.1
        )
    }
}

impl std::error::Error for ComponentNotInEntity {}
#[derive(Debug)]
pub enum ComponentError {
    ComponentNotInEntity(ComponentNotInEntity),
    EntityNotFound(EntityNotFound ),
}

#[derive(Debug)]
pub enum FetchError {
    ComponentAlreadyBorrowed(ComponentAlreadyBorrowed),
    ComponentDoesNotExist(ComponentDoesNotExist),
}

#[derive(Debug)]
pub struct ComponentAlreadyBorrowed(&'static str);

impl ComponentAlreadyBorrowed {
    pub fn new<T>() -> Self {
        Self(std::any::type_name::<T>())
    }
}
impl Default for ComponentAlreadyBorrowed{
    fn default() -> Self {
        Self("Component")
    }
}

impl std::fmt::Display for ComponentAlreadyBorrowed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] is already borrowed from the archetype", self.0)
    }
}

impl std::error::Error for ComponentAlreadyBorrowed {}

#[derive(Debug)]
pub struct ComponentDoesNotExist(&'static str);

impl ComponentDoesNotExist {
    pub fn new<T>() -> Self {
        Self(std::any::type_name::<T>())
    }
}
impl Default for ComponentDoesNotExist {
    fn default() -> Self {
        Self("Component")
    }
}
impl std::fmt::Display for ComponentDoesNotExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] does not exist", self.0)
    }
}

impl std::error::Error for ComponentDoesNotExist {}