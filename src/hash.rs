use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

pub trait MerkleHasher {
    type Hash: Clone + Copy + Eq;

    fn hash_leaf(data: &[u8]) -> Self::Hash;

    fn hash_children(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Hasher;

impl MerkleHasher for Sha256Hasher {
    type Hash = Hash;

    fn hash_leaf(data: &[u8]) -> Self::Hash {
        let mut hasher = Sha256::new();
        hasher.update([0x00]);
        hasher.update(data);
        hasher.finalize().into()
    }

    fn hash_children(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
        let mut hasher = Sha256::new();
        hasher.update([0x01]);
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}
