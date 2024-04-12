use std::fmt::Binary;

use super::errors::*;



#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PrimitiveId {
    Vertex(u32),
    Edge(u32),
    Face(u32),
    Unknown
}

impl Default for PrimitiveId {
    fn default() -> Self {
        PrimitiveId::Unknown
    }
}

impl PrimitiveId {
    pub fn vertex(self) -> Result<u32, IsNotVertexErr> {
        match self {
            PrimitiveId::Vertex(id) => Ok(id),
            _ => Err(IsNotVertexErr)
        }
    }
    pub fn edge(self) -> Result<u32, IsNotEdgeErr> {
        match self {
            PrimitiveId::Edge(id) => Ok(id),
            _ => Err(IsNotEdgeErr)
        }
    }
    pub fn face(self) -> Result<u32, IsNotFaceErr> {
        match self {
            PrimitiveId::Face(id) => Ok(id),
            _ => Err(IsNotFaceErr)
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct WrappedPrimitiveId(pub u32);

impl WrappedPrimitiveId {
    fn assert_mask<T>(code: T)
    where 
        T: std::fmt::Debug + std::ops::BitAnd<u32> + Binary + std::marker::Copy,
        <T as std::ops::BitAnd<u32>>::Output: PartialEq<u32>
    {
        if (code & Self::BIT_MASK) != 0u32 {
            panic!("Primitive does not have required flag '{:#032b}', actual value: {:#032b} ", Self::BIT_MASK, code);
        }
    }
    pub const UNKNOWN: Self = Self(0);

    const CODE_MASK: u32 = 0x3fff_ffff;
    const BIT_MASK: u32 = !Self::CODE_MASK;
    const VERTEX_BIT: u32 = 0b01 << 30;
    const EDGE_BIT: u32 = 0b10 << 30;
    const FACE_BIT: u32 = 0b11 << 30;

    pub fn vertex(code: u32) -> Self {
        Self::assert_mask(code);
        Self(Self::VERTEX_BIT | code)
    }

    pub fn edge(code: u32) -> Self {
        Self::assert_mask(code);
        Self(Self::EDGE_BIT | code)
    }

    pub fn face(code: u32) -> Self {
        Self::assert_mask(code);
        Self(Self::FACE_BIT | code)
    }

    pub fn vertices(code: [u32; 4]) -> [Self; 4] {
        [
            Self::vertex(code[0]),
            Self::vertex(code[1]),
            Self::vertex(code[2]),
            Self::vertex(code[3]),
        ]
    }

    pub fn edges(code: [u32; 4]) -> [Self; 4] {
        [
            Self::edge(code[0]),
            Self::edge(code[1]),
            Self::edge(code[2]),
            Self::edge(code[3]),
        ]
    }

    pub fn unpack(self) -> PrimitiveId {
        let bit = self.0 & Self::BIT_MASK;
        let code = self.0 & Self::CODE_MASK;
        match bit {
            Self::VERTEX_BIT => PrimitiveId::Vertex(code),
            Self::EDGE_BIT => PrimitiveId::Edge(code),
            Self::FACE_BIT => PrimitiveId::Face(code),
            _ => PrimitiveId::Unknown,
        }
    }

    pub fn is_face(self) -> bool {
        self.0 & Self::BIT_MASK == Self::FACE_BIT
    }

    pub fn is_edge(self) -> bool {
        self.0 & Self::BIT_MASK == Self::EDGE_BIT
    }
    pub fn is_vertex(self) -> bool {
        self.0 & Self::BIT_MASK == Self::VERTEX_BIT
    }
    
    pub fn is_unknown(self) -> bool {
        self == Self::UNKNOWN
    }
}

impl From<PrimitiveId> for WrappedPrimitiveId {
    fn from(value: PrimitiveId) -> Self {
        match value {
            PrimitiveId::Face(primitive_id) => Self::face(primitive_id),
            PrimitiveId::Edge(primitive_id) => Self::edge(primitive_id),
            PrimitiveId::Vertex(primitive_id) => Self::vertex(primitive_id),
            PrimitiveId::Unknown => Self::UNKNOWN,
        }
    }
}
impl Default for WrappedPrimitiveId {
    fn default() -> Self {
        WrappedPrimitiveId::UNKNOWN
    }
}
impl From<usize> for WrappedPrimitiveId {
    fn from(value: usize) -> Self {
        WrappedPrimitiveId(value as u32)
    }
}