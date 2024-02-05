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