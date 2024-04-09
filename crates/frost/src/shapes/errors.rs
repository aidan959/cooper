use std::fmt;
#[derive(Debug, Clone)]
pub struct IsNotVertexErr;

impl fmt::Display for IsNotVertexErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The primitive id is not a vertex.")
    }
}

#[derive(Debug, Clone)]
pub struct IsNotEdgeErr;

impl fmt::Display for IsNotEdgeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The primitive id is not an edge.")
    }
}
#[derive(Debug, Clone)]
pub struct IsNotFaceErr;

impl fmt::Display for IsNotFaceErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The primitive id is not a face.")
    }
}

