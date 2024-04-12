use std::sync::atomic::AtomicI64;
use std::hash::Hash;
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    generation: u32,
    index: u32,
}

impl Hash for Entity {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl Entity {
    #[cfg(test)]
    pub(crate) const fn new(index: u32, generation: u32) -> Entity {
        Entity { index, generation }
    }

    pub const PLACEHOLDER: Self = Self::from_raw(u32::MAX);

    pub const fn from_raw(index: u32) -> Entity {
        Entity {
            index,
            generation: 0,
        }
    }

    pub const fn to_bits(self) -> u64 {
        (self.generation as u64) << 32 | self.index as u64
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self {
            generation: (bits >> 32) as u32,
            index: bits as u32,
        }
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.index
    }
}

#[derive(Debug)]
pub struct Entities {
    //meta: Vec<EntityMeta>,
    pending: Vec<u32>,
    free_cursor: AtomicI64,
    /// Stores the number of free entities for [`len`](Entities::len) 
    len: u32,
}