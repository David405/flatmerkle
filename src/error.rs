use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MerkleError {
    EmptyTree,
    IndexOutOfBounds { index: usize, leaf_count: usize },
}

impl fmt::Display for MerkleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTree => write!(f, "cannot build a Merkle tree from zero leaves"),
            Self::IndexOutOfBounds { index, leaf_count } => {
                write!(
                    f,
                    "leaf index {index} is out of bounds for tree with {leaf_count} leaves"
                )
            }
        }
    }
}

impl std::error::Error for MerkleError {}
