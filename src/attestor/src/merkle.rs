use sha2::{Digest, Sha256};

/// SHA-256 hash type used throughout the Merkle tree.
pub type Hash = [u8; 32];

const LEAF_DOMAIN: &[u8] = b"runtimeguard/merkle-leaf/v2";
const NODE_DOMAIN: &[u8] = b"runtimeguard/merkle-node/v2";

/// Domain-separated Merkle tree with duplicate-last handling for odd levels.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Internal storage of tree levels. Level 0 = leaves, Level n = root.
    levels: Vec<Vec<Hash>>,
    leaf_count: usize,
}

impl MerkleTree {
    /// Builds a Merkle tree from raw leaf data.
    pub fn from_data(data: &[&[u8]]) -> Self {
        if data.is_empty() {
            return Self {
                levels: vec![vec![]],
                leaf_count: 0,
            };
        }

        let leaf_count = data.len();
        let mut leaves: Vec<Hash> = data.iter().map(|item| hash_leaf(item)).collect();
        if leaves.len() % 2 == 1 {
            leaves.push(*leaves.last().expect("non-empty leaves"));
        }

        let mut levels = vec![leaves];
        while levels.last().expect("at least one level").len() > 1 {
            let current = levels.last().expect("at least one level");
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            for pair in current.chunks(2) {
                let right = pair.get(1).unwrap_or(&pair[0]);
                next.push(hash_node(&pair[0], right));
            }
            levels.push(next);
        }

        Self { levels, leaf_count }
    }

    /// Returns the root hash, or `None` for an empty tree.
    pub fn root(&self) -> Option<Hash> {
        self.levels.last().and_then(|level| level.first()).copied()
    }

    /// Generates a sibling path for a logical leaf index.
    pub fn generate_proof(&self, leaf_index: usize) -> Vec<Hash> {
        if leaf_index >= self.leaf_count {
            return Vec::new();
        }
        let mut proof = Vec::new();
        let mut index = leaf_index;
        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling = if index.is_multiple_of(2) {
                index + 1
            } else {
                index - 1
            };
            proof.push(*level.get(sibling).unwrap_or(&level[index]));
            index /= 2;
        }
        proof
    }

    /// Legacy verifier retained for low-level tests. Protocol verification must
    /// use `verify_proof_with_size` so that the signed logical tree size is bound.
    pub fn verify_proof(data: &[u8], index: usize, proof: &[Hash], root: &Hash) -> bool {
        let mut current_hash = hash_leaf(data);
        let mut current_index = index;
        for sibling in proof {
            current_hash = if current_index.is_multiple_of(2) {
                hash_node(&current_hash, sibling)
            } else {
                hash_node(sibling, &current_hash)
            };
            current_index /= 2;
        }
        &current_hash == root
    }

    /// Verifies proof shape for a caller-supplied logical tree size and index.
    /// The signed epoch statement must separately bind `leaf_count` because a
    /// duplicate-last tree can have the same root for adjacent logical sizes.
    pub fn verify_proof_with_size(
        data: &[u8],
        index: usize,
        leaf_count: usize,
        proof: &[Hash],
        root: &Hash,
    ) -> bool {
        if leaf_count == 0 || index >= leaf_count || proof.len() != proof_depth(leaf_count) {
            return false;
        }
        Self::verify_proof(data, index, proof, root)
    }
}

fn proof_depth(leaf_count: usize) -> usize {
    let mut width = if leaf_count.is_multiple_of(2) {
        leaf_count
    } else {
        leaf_count + 1
    };
    let mut depth = 0;
    while width > 1 {
        width = width.div_ceil(2);
        depth += 1;
    }
    depth
}

fn hash_leaf(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(LEAF_DOMAIN);
    hasher.update((data.len() as u64).to_be_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

fn hash_node(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(NODE_DOMAIN);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_leaf_proofs_verify_for_every_logical_leaf() {
        let items = [b"a".as_slice(), b"b", b"c", b"d", b"e"];
        let tree = MerkleTree::from_data(&items);
        let root = tree.root().expect("root");

        for (index, item) in items.iter().enumerate() {
            let proof = tree.generate_proof(index);
            assert!(MerkleTree::verify_proof_with_size(
                item,
                index,
                items.len(),
                &proof,
                &root
            ));
        }
    }

    #[test]
    fn proof_rejects_wrong_data_index_or_tree_size() {
        let items = [b"a".as_slice(), b"b", b"c"];
        let tree = MerkleTree::from_data(&items);
        let root = tree.root().expect("root");
        let proof = tree.generate_proof(2);

        assert!(!MerkleTree::verify_proof_with_size(
            b"tampered",
            2,
            items.len(),
            &proof,
            &root
        ));
        assert!(!MerkleTree::verify_proof_with_size(
            b"c",
            1,
            items.len(),
            &proof,
            &root
        ));
        assert!(!MerkleTree::verify_proof_with_size(
            b"c", 2, 8, &proof, &root
        ));
    }

    #[test]
    fn singleton_tree_has_a_size_bound_duplicate_proof() {
        let items = [b"a".as_slice()];
        let tree = MerkleTree::from_data(&items);
        let root = tree.root().expect("root");
        let proof = tree.generate_proof(0);
        assert_eq!(proof.len(), 1);
        assert!(MerkleTree::verify_proof_with_size(
            b"a", 0, 1, &proof, &root
        ));
    }

    #[test]
    fn empty_tree_has_no_root_or_proof() {
        let tree = MerkleTree::from_data(&[]);
        assert_eq!(tree.root(), None);
        assert!(tree.generate_proof(0).is_empty());
    }

    #[test]
    fn leaf_and_internal_domains_do_not_alias() {
        let payload = [7_u8; 64];
        let left: Hash = payload[..32].try_into().expect("left hash");
        let right: Hash = payload[32..].try_into().expect("right hash");
        assert_ne!(hash_leaf(&payload), hash_node(&left, &right));
    }
}
