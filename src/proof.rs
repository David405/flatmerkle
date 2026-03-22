#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiblingPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStep<H> {
    pub sibling: H,
    pub position: SiblingPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof<H> {
    pub leaf_index: usize,
    pub leaf_count: usize,
    pub steps: Vec<ProofStep<H>>,
}
