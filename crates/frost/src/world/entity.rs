use std::{
    fmt,
    num::NonZeroI32,
    sync::atomic::{AtomicUsize, Ordering},
};
//pub type Entities<'a> = Read<'a, EntitiesRes>;
pub type Index = u32;
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity(Index, Generation);

impl Entity {
    /// Creates a new entity (externally from ECS).
    #[cfg(test)]
    pub fn new(index: Index, gen: Generation) -> Self {
        Self(index, gen)
    }

    /// Returns the index of the `Entity`.
    #[inline]
    pub fn id(self) -> Index {
        self.0
    }

    /// Returns the `Generation` of the `Entity`.
    #[inline]
    pub fn gen(self) -> Generation {
        self.1
    }
}

#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Generation(NonZeroI32);

impl Generation {
    pub(crate) fn one() -> Self {
        // SAFETY: `1` is not zero.
        Generation(unsafe { NonZeroI32::new_unchecked(1) })
    }

    #[cfg(test)]
    pub fn new(v: i32) -> Self {
        Generation(NonZeroI32::new(v).expect("generation id must be non-zero"))
    }

    /// Returns the id of the generation.
    #[inline]
    pub fn id(self) -> i32 {
        self.0.get()
    }

    /// Returns `true` if entities of this `Generation` are alive.
    #[inline]
    pub fn is_alive(self) -> bool {
        self.id() > 0
    }

    /// Revives and increments a dead `Generation`.
    ///
    /// # Panics
    ///
    /// Panics if it is alive.
    fn raised(self) -> Generation {
        assert!(!self.is_alive());
        // SAFETY: Since `self` is not alive, `self.id()` will be negative so
        // subtracting it from `1` will give us a value `>= 2`. If this
        // overflows it will at most wrap to `i32::MIN + 1` (so it will never be
        // zero).
        unsafe { Generation(NonZeroI32::new_unchecked(1 - self.id())) }
    }
}