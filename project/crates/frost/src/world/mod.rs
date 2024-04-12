use std::{sync::{atomic::AtomicUsize}, num::NonZeroI32};

use hibitset::{BitSet, AtomicBitSet};
use shred::{World as ShredWorld};
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Generation(NonZeroI32);
impl Generation {
    pub fn one() -> Self {
        Generation(NonZeroI32::new(1).unwrap())
    }
    pub fn new(value: i32) -> Option<Self> {
        if let Some(val) = NonZeroI32::new(value) {
            Some(Generation(val))
        } else {
            None
        }
    }

    pub fn id(self) -> i32 {
        self.0.get()
    }
}
impl std::fmt::Debug for Generation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Generation").field(&self.0.get()).finish()
    }
}
#[derive(Default, Debug)]
struct ZeroableGeneration(Option<Generation>);
#[derive(Default, Debug)]
struct EntityCache {
    cache: Vec<u32>,
    len: AtomicUsize,
}
#[derive(Debug, Default)]
pub(crate) struct Allocator {
    generations: Vec<ZeroableGeneration>,
    alive: BitSet,
    raised: AtomicBitSet,
    killed: AtomicBitSet,
    cache: EntityCache,
    max_id: AtomicUsize,
}
#[derive(Debug, Default)]
pub struct EntitiesRes {
    pub(crate) alloc: Allocator,
}
trait World {
    fn new() -> Self;
}
impl World for ShredWorld {
    fn new() -> Self{
        let mut world = Self::default();
        // world.insert(MetaTable::<dyn AnyStorage>::default());
        world.insert(EntitiesRes::default());
        world
    }
}
struct SysA;

// impl<'a> System<'a> for SysA {
//     type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

//     fn run(&mut self, (mut pos, vel): Self::SystemData) {
//         for (pos, vel) in (&mut pos, &vel).join() {
//             pos.0 += vel.0;
//         }
//     }
// }
#[cfg(test)]
mod tests {
    use shred::DispatcherBuilder;

    use super::World;

    #[test]
    fn create_world() {
        let mut world = World::new();

        //let mut dispatcher = DispatcherBuilder::new().with(SysA, name, dep);

    }
}
