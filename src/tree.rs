use crate::error::MerkleError;
use crate::hash::MerkleHasher;
use crate::proof::{MerkleProof, ProofStep, SiblingPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OddNodePolicy {
    #[default]
    DuplicateLast,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatMerkleTree<H> {
    nodes: Vec<H>,
    level_offsets: Vec<usize>,
    leaf_count: usize,
    odd_node_policy: OddNodePolicy,
}

impl<H> FlatMerkleTree<H>
where
    H: Clone + Copy + Eq,
{
    pub fn from_leaves<T, M>(leaves: &[T]) -> Result<Self, MerkleError>
    where
        T: AsRef<[u8]>,
        M: MerkleHasher<Hash = H>,
    {
        Self::from_leaves_with_policy::<T, M>(leaves, OddNodePolicy::DuplicateLast)
    }

    pub fn from_leaves_with_policy<T, M>(
        leaves: &[T],
        odd_node_policy: OddNodePolicy,
    ) -> Result<Self, MerkleError>
    where
        T: AsRef<[u8]>,
        M: MerkleHasher<Hash = H>,
    {
        if leaves.is_empty() {
            return Err(MerkleError::EmptyTree);
        }

        let mut current_level: Vec<H> = leaves
            .iter()
            .map(|leaf| M::hash_leaf(leaf.as_ref()))
            .collect();
        let mut nodes = Vec::with_capacity(total_node_count(current_level.len()));
        let mut level_offsets = Vec::new();

        loop {
            level_offsets.push(nodes.len());
            nodes.extend_from_slice(&current_level);

            if current_level.len() == 1 {
                break;
            }

            current_level = build_parent_level::<M>(&current_level, odd_node_policy);
        }

        Ok(Self {
            nodes,
            level_offsets,
            leaf_count: leaves.len(),
            odd_node_policy,
        })
    }

    pub fn root(&self) -> &H {
        self.nodes
            .last()
            .expect("FlatMerkleTree is only constructed from non-empty leaves")
    }

    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    pub fn odd_node_policy(&self) -> OddNodePolicy {
        self.odd_node_policy
    }

    pub fn leaf_hash(&self, index: usize) -> Option<&H> {
        self.level(0).get(index)
    }

    pub fn proof(&self, leaf_index: usize) -> Result<MerkleProof<H>, MerkleError> {
        if leaf_index >= self.leaf_count {
            return Err(MerkleError::IndexOutOfBounds {
                index: leaf_index,
                leaf_count: self.leaf_count,
            });
        }

        let mut index = leaf_index;
        let mut steps = Vec::with_capacity(self.level_offsets.len().saturating_sub(1));

        for level_index in 0..self.level_offsets.len().saturating_sub(1) {
            let level = self.level(level_index);
            let sibling_index = if index % 2 == 0 {
                if index + 1 < level.len() {
                    index + 1
                } else {
                    index
                }
            } else {
                index - 1
            };

            let position = if sibling_index < index {
                SiblingPosition::Left
            } else {
                SiblingPosition::Right
            };

            steps.push(ProofStep {
                sibling: level[sibling_index],
                position,
            });

            index /= 2;
        }

        Ok(MerkleProof {
            leaf_index,
            leaf_count: self.leaf_count,
            steps,
        })
    }

    fn level(&self, level_index: usize) -> &[H] {
        let start = self.level_offsets[level_index];
        let end = self
            .level_offsets
            .get(level_index + 1)
            .copied()
            .unwrap_or(self.nodes.len());
        &self.nodes[start..end]
    }
}

pub fn verify_proof<M>(
    leaf_data: &[u8],
    proof: &MerkleProof<M::Hash>,
    expected_root: &M::Hash,
) -> bool
where
    M: MerkleHasher,
{
    if proof.leaf_count == 0 || proof.leaf_index >= proof.leaf_count {
        return false;
    }

    let mut current = M::hash_leaf(leaf_data);

    for step in &proof.steps {
        current = match step.position {
            SiblingPosition::Left => M::hash_children(&step.sibling, &current),
            SiblingPosition::Right => M::hash_children(&current, &step.sibling),
        };
    }

    &current == expected_root
}

fn build_parent_level<M>(level: &[M::Hash], odd_node_policy: OddNodePolicy) -> Vec<M::Hash>
where
    M: MerkleHasher,
{
    let mut parents = Vec::with_capacity(level.len().div_ceil(2));

    let mut chunks = level.chunks_exact(2);
    for pair in &mut chunks {
        parents.push(M::hash_children(&pair[0], &pair[1]));
    }

    let remainder = chunks.remainder();
    if let Some(last) = remainder.first() {
        match odd_node_policy {
            OddNodePolicy::DuplicateLast => parents.push(M::hash_children(last, last)),
        }
    }

    parents
}

fn total_node_count(mut level_len: usize) -> usize {
    let mut total = 0;
    while level_len > 0 {
        total += level_len;
        if level_len == 1 {
            break;
        }
        level_len = level_len.div_ceil(2);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{verify_proof, FlatMerkleTree, OddNodePolicy};
    use crate::error::MerkleError;
    use crate::hash::{Hash, MerkleHasher};

    #[derive(Debug, Clone, Copy)]
    struct TestHasher;

    impl MerkleHasher for TestHasher {
        type Hash = Hash;

        fn hash_leaf(data: &[u8]) -> Self::Hash {
            let mut out = [0_u8; 32];
            out[0] = data.iter().fold(0_u8, |acc, byte| acc.wrapping_add(*byte));
            out[1] = data.len() as u8;
            out
        }

        fn hash_children(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
            let mut out = [0_u8; 32];
            out[0] = left[0].wrapping_add(right[0]).wrapping_add(1);
            out[1] = left[1].wrapping_add(right[1]).wrapping_add(2);
            out
        }
    }

    fn sample_leaves() -> Vec<Vec<u8>> {
        vec![
            b"a".to_vec(),
            b"b".to_vec(),
            b"c".to_vec(),
            b"d".to_vec(),
            b"e".to_vec(),
            b"f".to_vec(),
            b"g".to_vec(),
            b"h".to_vec(),
        ]
    }

    #[test]
    fn rejects_empty_tree() {
        let leaves: Vec<Vec<u8>> = Vec::new();
        let error = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap_err();
        assert_eq!(error, MerkleError::EmptyTree);
    }

    #[test]
    fn single_leaf_tree_root_matches_leaf_hash() {
        let leaves = vec![b"only".to_vec()];
        let tree = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap();

        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(*tree.root(), TestHasher::hash_leaf(b"only"));
    }

    #[test]
    fn duplicate_last_policy_is_used_for_odd_leaf_count() {
        let leaves = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let tree = FlatMerkleTree::<Hash>::from_leaves_with_policy::<_, TestHasher>(
            &leaves,
            OddNodePolicy::DuplicateLast,
        )
        .unwrap();

        let a = TestHasher::hash_leaf(b"a");
        let b = TestHasher::hash_leaf(b"b");
        let c = TestHasher::hash_leaf(b"c");
        let left = TestHasher::hash_children(&a, &b);
        let right = TestHasher::hash_children(&c, &c);
        let expected_root = TestHasher::hash_children(&left, &right);

        assert_eq!(tree.odd_node_policy(), OddNodePolicy::DuplicateLast);
        assert_eq!(*tree.root(), expected_root);
    }

    #[test]
    fn proof_verifies_for_first_middle_and_last_leaf() {
        let leaves = sample_leaves();
        let tree = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap();

        for index in [0, 3, leaves.len() - 1] {
            let proof = tree.proof(index).unwrap();
            assert!(verify_proof::<TestHasher>(
                &leaves[index],
                &proof,
                tree.root()
            ));
        }
    }

    #[test]
    fn proof_rejects_wrong_leaf_data() {
        let leaves = sample_leaves();
        let tree = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap();
        let proof = tree.proof(2).unwrap();

        assert!(!verify_proof::<TestHasher>(b"wrong", &proof, tree.root()));
    }

    #[test]
    fn proof_rejects_out_of_bounds_index() {
        let leaves = sample_leaves();
        let tree = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap();

        let error = tree.proof(leaves.len()).unwrap_err();
        assert_eq!(
            error,
            MerkleError::IndexOutOfBounds {
                index: leaves.len(),
                leaf_count: leaves.len(),
            }
        );
    }

    #[test]
    fn leaf_hash_returns_expected_values() {
        let leaves = sample_leaves();
        let tree = FlatMerkleTree::<Hash>::from_leaves::<_, TestHasher>(&leaves).unwrap();

        assert_eq!(tree.leaf_hash(0), Some(&TestHasher::hash_leaf(b"a")));
        assert_eq!(tree.leaf_hash(leaves.len()), None);
    }
}
