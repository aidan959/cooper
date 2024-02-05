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
