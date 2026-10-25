//! Poseidon sparse-Merkle commitment tree.
//!
//! Sparse, fixed depth, with an implicit default leaf of `0`. Fixed depth keeps
//! the circuit a fixed shape, which is what allows one verifying key to serve
//! every proof. A registry of three records and a registry of three million
//! produce the same proof size.
//!
//! The tree exists only off chain. The chain receives a root from governance and
//! a proof from a citizen; it never builds or walks a tree.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::poseidon::{
    find_poseidon_ark_and_mds, PoseidonConfig, PoseidonSponge,
};
use ark_crypto_primitives::sponge::{CryptographicSponge, FieldBasedCryptographicSponge};
use ark_ff::Zero;

/// Fixed tree depth. 2^32 leaves is far beyond any Belizean registry.
pub const TREE_DEPTH: usize = 32;

/// Rate of the Poseidon sponge used as a 2-to-1 compression.
const RATE: usize = 2;
const CAPACITY: usize = 1;
const FULL_ROUNDS: u64 = 8;
const PARTIAL_ROUNDS: u64 = 57;
/// S-box exponent. Must be coprime with `r - 1`; 5 is the standard choice for
/// BLS12-381 and is what makes the gadget's `pow_by_constant` cheap.
const ALPHA: u64 = 5;

/// The Poseidon parameters, generated deterministically.
///
/// Determinism matters more than it looks: the prover and the verifier must
/// agree on these constants or every proof fails, and there is no compile-time
/// check for that. Both sides call this one function.
///
/// Generation is not cheap (it runs a Grain LFSR to derive the round constants),
/// so callers should generate once and pass the reference around rather than
/// calling it per hash.
pub fn poseidon_config() -> PoseidonConfig<Fr> {
    // `prime_bits` for BLS12-381's scalar field: 255.
    let (ark, mds) = find_poseidon_ark_and_mds::<Fr>(
        255,
        RATE,
        FULL_ROUNDS,
        PARTIAL_ROUNDS,
        0, // skip_matrices: 0 keeps the standard first row/column constraints
    );
    PoseidonConfig {
        full_rounds: FULL_ROUNDS as usize,
        partial_rounds: PARTIAL_ROUNDS as usize,
        alpha: ALPHA,
        ark,
        mds,
        rate: RATE,
        capacity: CAPACITY,
    }
}

/// Poseidon over exactly two field elements.
///
/// Absorbs one element at a time so the sequence is unambiguous — the gadget
/// does the same, and matching call order is the only thing keeping the two
/// implementations in agreement.
pub fn hash_two(params: &PoseidonConfig<Fr>, left: Fr, right: Fr) -> Fr {
    let mut sponge = PoseidonSponge::<Fr>::new(params);
    sponge.absorb(&left);
    sponge.absorb(&right);
    sponge.squeeze_native_field_elements(1)[0]
}

/// Poseidon over an arbitrary list, absorbed in order.
pub fn hash_many(params: &PoseidonConfig<Fr>, values: &[Fr]) -> Fr {
    let mut sponge = PoseidonSponge::<Fr>::new(params);
    for value in values {
        sponge.absorb(value);
    }
    sponge.squeeze_native_field_elements(1)[0]
}

/// `record_hash = Poseidon(subject, value, tag)`.
///
/// Separating this from the leaf hash means a leaf commits to a record through
/// one intermediate value, so changing the record shape changes this function
/// only.
pub fn record_hash(params: &PoseidonConfig<Fr>, subject: Fr, value: Fr, tag: Fr) -> Fr {
    hash_many(params, &[subject, value, tag])
}

/// `leaf = Poseidon(record_hash, salt)`.
pub fn leaf_hash(params: &PoseidonConfig<Fr>, record: Fr, salt: Fr) -> Fr {
    hash_two(params, record, salt)
}

/// A sparse Merkle tree over `Fr` leaves.
#[derive(Clone)]
pub struct SparseTree {
    depth: usize,
    /// Only occupied leaves are stored; absent indices read as zero.
    leaves: BTreeMap<u64, Fr>,
    /// Empty-subtree roots, indexed by height. `empty[0]` is the zero leaf.
    empty: Vec<Fr>,
}

impl SparseTree {
    /// Build a tree from `(index, leaf)` pairs at the default depth.
    ///
    /// Returns an error if two entries share an index: silently keeping the last
    /// would make the published root disagree with what the caller believes it
    /// published.
    pub fn new(
        params: &PoseidonConfig<Fr>,
        entries: impl IntoIterator<Item = (u64, Fr)>,
    ) -> Result<Self, TreeError> {
        Self::with_depth(params, TREE_DEPTH, entries)
    }

    pub fn with_depth(
        params: &PoseidonConfig<Fr>,
        depth: usize,
        entries: impl IntoIterator<Item = (u64, Fr)>,
    ) -> Result<Self, TreeError> {
        if depth == 0 || depth > 63 {
            return Err(TreeError::BadDepth(depth));
        }

        let mut empty = Vec::with_capacity(depth + 1);
        empty.push(Fr::zero());
        for level in 1..=depth {
            let previous = empty[level - 1];
            empty.push(hash_two(params, previous, previous));
        }

        let mut leaves = BTreeMap::new();
        for (index, leaf) in entries {
            if index >= (1u64 << depth) {
                return Err(TreeError::IndexOutOfRange { index, depth });
            }
            if leaves.insert(index, leaf).is_some() {
                return Err(TreeError::DuplicateIndex(index));
            }
        }

        Ok(Self { depth, leaves, empty })
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Root of the whole tree.
    pub fn root(&self, params: &PoseidonConfig<Fr>) -> Fr {
        self.subtree_root(params, self.depth, 0)
    }

    /// Root of the subtree of height `height` whose leaf range starts at `base`.
    ///
    /// `base` must have its low `height` bits clear; callers inside this type
    /// always satisfy that.
    fn subtree_root(&self, params: &PoseidonConfig<Fr>, height: usize, base: u64) -> Fr {
        if height == 0 {
            return self.leaves.get(&base).copied().unwrap_or_else(Fr::zero);
        }
        if !self.has_any_leaf_under(height, base) {
            return self.empty[height];
        }
        let half = 1u64 << (height - 1);
        let left = self.subtree_root(params, height - 1, base);
        let right = self.subtree_root(params, height - 1, base | half);
        hash_two(params, left, right)
    }

    /// True when at least one occupied leaf falls inside the range covered by
    /// `(height, base)`. Lets an entirely empty subtree return its precomputed
    /// root instead of walking `2^depth` nodes.
    fn has_any_leaf_under(&self, height: usize, base: u64) -> bool {
        let span = 1u64 << height;
        self.leaves
            .range(base..base.saturating_add(span))
            .next()
            .is_some()
    }

    /// Sibling hashes for `index`, lowest level first.
    ///
    /// Level `L` holds the root of the subtree that shares the low `L` bits with
    /// `index` and differs at bit `L` — i.e. the node `index` is hashed against.
    pub fn path(&self, params: &PoseidonConfig<Fr>, index: u64) -> Result<Vec<Fr>, TreeError> {
        if index >= (1u64 << self.depth) {
            return Err(TreeError::IndexOutOfRange { index, depth: self.depth });
        }
        let mut siblings = Vec::with_capacity(self.depth);
        for level in 0..self.depth {
            // Flip bit `level` to reach the sibling subtree.
            let sibling_index = index ^ (1u64 << level);
            // Clear all bits below `level` so it is a valid subtree base.
            let base = sibling_index & !((1u64 << level) - 1);
            siblings.push(self.subtree_root(params, level, base));
        }
        Ok(siblings)
    }
}

/// Recompute a root from a leaf and its path — the same walk the circuit does.
pub fn root_from_path(
    params: &PoseidonConfig<Fr>,
    leaf: Fr,
    index: u64,
    path: &[Fr],
) -> Result<Fr, TreeError> {
    if path.len() > 63 {
        return Err(TreeError::BadDepth(path.len()));
    }
    let mut current = leaf;
    for (level, sibling) in path.iter().enumerate() {
        current = if (index >> level) & 1 == 0 {
            // `current` sits on the left, so it is hashed first.
            hash_two(params, current, *sibling)
        } else {
            hash_two(params, *sibling, current)
        };
    }
    Ok(current)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TreeError {
    DuplicateIndex(u64),
    IndexOutOfRange { index: u64, depth: usize },
    BadDepth(usize),
}

impl core::fmt::Display for TreeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateIndex(index) => write!(f, "duplicate leaf index {index}"),
            Self::IndexOutOfRange { index, depth } => {
                write!(f, "leaf index {index} does not fit depth {depth}")
            }
            Self::BadDepth(depth) => write!(f, "unsupported tree depth {depth}"),
        }
    }
}
