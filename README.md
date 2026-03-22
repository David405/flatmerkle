# flatmerkle

`flatmerkle` is a Rust library for building Merkle trees in a flat contiguous memory layout.

Instead of representing the tree as a graph of heap-allocated nodes, `flatmerkle` stores each level in a single backing vector with offset metadata. That makes the implementation easier to reason about, friendlier to cache locality, and better suited for proof-heavy workloads.

The crate currently provides:

- Flat Merkle tree construction
- Root computation
- Inclusion proof generation
- Inclusion proof verification
- Generic hashing via a `MerkleHasher` trait
- A default `Sha256Hasher`
- Deterministic odd-node handling with duplicate-last semantics

## Why FlatMerkle

Traditional pointer-based Merkle trees are simple to explain, but they come with overhead:

- many heap allocations
- pointer chasing during traversal
- less predictable memory access patterns
- more bookkeeping for proof generation

`flatmerkle` is designed around a different tradeoff:

- contiguous storage for tree nodes
- compact internal representation
- deterministic proof structure
- a small public API that is easy to integrate into other systems

This makes it a good fit for systems that need to build trees often, generate many proofs, or keep memory behavior easy to inspect and benchmark.

## Primary Use Cases

### Blockchain and rollup infrastructure

Merkle roots are widely used to commit to transaction sets, account snapshots, state transitions, and data batches. A flat in-memory representation is useful when you need to compute roots and generate proofs repeatedly inside sequencers, indexers, provers, or block builders.

### Data availability and integrity proofs

If you need to prove that a file chunk, record, or payload belongs to a committed dataset, `flatmerkle` provides a straightforward way to compute a root and hand out inclusion proofs for later verification.

### Indexing and archival systems

Off-chain systems often need compact commitment layers over ordered records. `flatmerkle` can be used to commit to batches of rows, logs, or snapshots so downstream consumers can verify membership without trusting the full dataset.

### Distributed systems and replication

Merkle trees are useful for comparing replicas, validating snapshots, and establishing trust boundaries between nodes. A flat layout is especially attractive in high-throughput services where allocation patterns matter.

### Research and benchmarking

This project is also a useful base for studying how memory layout affects Merkle tree performance. The current implementation is structured so it can later be compared with a pointer-based baseline in reproducible benchmarks.

## Current Status

The crate is functional but still early-stage.

What is implemented today:

- flat tree construction
- root retrieval
- proof generation
- proof verification
- basic error handling
- unit tests for core and edge-case behavior

What is planned next:

- a pointer-based baseline for comparison
- Criterion benchmark harnesses
- expanded integration tests
- examples and richer crate documentation

## Installation

This crate is not yet published on crates.io.

For local development, use it from a path dependency:

```toml
[dependencies]
flatmerkle = { path = "../flatmerkle" }
```

## Quick Start

```rust
use flatmerkle::{verify_proof, FlatMerkleTree, Sha256Hasher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let leaves = vec![
        b"alice".to_vec(),
        b"bob".to_vec(),
        b"carol".to_vec(),
        b"dave".to_vec(),
    ];

    let tree = FlatMerkleTree::from_leaves::<_, Sha256Hasher>(&leaves)?;
    let root = *tree.root();

    let proof = tree.proof(1)?;
    let verified = verify_proof::<Sha256Hasher>(&leaves[1], &proof, &root);

    assert!(verified);
    Ok(())
}
```

## API Overview

### Build a tree

```rust
use flatmerkle::{FlatMerkleTree, Sha256Hasher};

let leaves = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
let tree = FlatMerkleTree::from_leaves::<_, Sha256Hasher>(&leaves)?;
```

### Read the root

```rust
let root = tree.root();
```

### Generate a proof

```rust
let proof = tree.proof(0)?;
```

### Verify a proof

```rust
use flatmerkle::{verify_proof, Sha256Hasher};

let ok = verify_proof::<Sha256Hasher>(&leaves[0], &proof, tree.root());
assert!(ok);
```

## Hashing Model

`flatmerkle` supports custom hashing through the `MerkleHasher` trait:

```rust
pub trait MerkleHasher {
    type Hash: Clone + Copy + Eq;

    fn hash_leaf(data: &[u8]) -> Self::Hash;
    fn hash_children(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
}
```

The default `Sha256Hasher` uses domain separation:

- leaf hashing prefixes data with `0x00`
- internal-node hashing prefixes children with `0x01`

This helps avoid accidental collisions between leaf and internal-node hashing domains.

## Tree Semantics

The current implementation uses the following semantics:

- leaves are hashed before insertion into the tree
- internal nodes are derived level by level
- odd-width levels duplicate the final node
- proofs carry sibling hashes and their left/right position
- empty trees are rejected

These rules are important because Merkle proofs are only meaningful when the builder and verifier agree on exactly how trees are constructed.

## Error Handling

The crate currently exposes a small error surface:

- `MerkleError::EmptyTree`
- `MerkleError::IndexOutOfBounds`

This keeps call sites simple while the library is still stabilizing.

## Example Use in a Service

One practical flow looks like this:

1. Collect a batch of records or payloads.
2. Build a `FlatMerkleTree` from the ordered leaf bytes.
3. Publish or store the root as the commitment.
4. Generate proofs for any leaf a client may need to verify.
5. Verify proofs later against the published root.

This pattern works well for:

- batch commitments
- API verifiability layers
- audit logs
- snapshot distribution
- integrity checks between services

## Performance Direction

The design of this crate is intentionally oriented toward:

- fewer heap objects
- more linear memory access
- easier benchmarking of structural overhead

That said, performance claims are still a work in progress until the benchmark suite and baseline comparison are added. The current README describes the intended direction, not a finalized benchmark result.

## Development

Run the current test suite with:

```bash
cargo test
```

Format the code with:

```bash
cargo fmt
```

## Roadmap

- Add a pointer-based baseline implementation
- Add Criterion benches for build, proof generation, and verification
- Add integration tests with deterministic fixtures
- Add examples and public API docs
- Add benchmark result artifacts and reproducibility notes

## License

Licensed under the Apache License 2.0. See [LICENSE-APACHE](/Users/davidasamonye/Work/david405/flatmerkle/LICENSE-APACHE).
