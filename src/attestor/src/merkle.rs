use sha2::{Digest, Sha256};

/// A Hash is a 32-byte SHA-256 output.
pub type Hash = [u8; 32];

/// A Merkle Tree implementation for RuntimeGuard-AI compliance logs.
/// 
/// This structure is used to aggregate compliance records and generate
/// tamper-evident Merkle Roots for ZK attestation (Paper Appendix B.2).
pub struct MerkleTree {
    /// Internal storage of tree levels. Level 0 = leaves, Level n = root.
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Constructs a new Merkle Tree from a list of data blocks.
    /// 
    /// Each block is hashed using SHA-256 to form the leaves.
    /// If the number of leaves is odd, the last leaf is duplicated.
    pub fn from_data(data: &[&[u8]]) -> Self {
        if data.is_empty() {
            return Self { levels: vec![vec![]] };
        }

        // Hash all data blocks to form leaves
        let mut leaves: Vec<Hash> = data
            .iter()
            .map(|block| {
                let mut hasher = Sha256::new();
                hasher.update(block);
                hasher.finalize().into()
            })
            .collect();

        // Ensure even number of leaves
        if leaves.len() % 2 != 0 {
            leaves.push(*leaves.last().unwrap());
        }

        let mut levels = vec![leaves];

        // Build tree levels until we reach the root
        while levels.last().unwrap().len() > 1 {
            let current_level = levels.last().unwrap();
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);

            for chunk in current_level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate for odd
                }
                next_level.push(hasher.finalize().into());
            }

            levels.push(next_level);
        }

        Self { levels }
    }

    /// Returns the Merkle Root (the hash at the top of the tree).
    pub fn root(&self) -> Option<Hash> {
        self.levels.last().and_then(|level| level.first().copied())
    }

    /// Generates a Merkle Inclusion Proof for a specific leaf index.
    /// 
    /// The proof consists of the sibling hashes along the path from leaf to root.
    /// Verification time is O(log n).
    /// 
    /// # Arguments
    /// * `leaf_index` - The 0-indexed position of the leaf.
    /// 
    /// # Returns
    /// A vector of sibling hashes from leaf to root.
    pub fn generate_proof(&self, leaf_index: usize) -> Vec<Hash> {
        let mut proof = Vec::new();
        let mut current_idx = leaf_index;

        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling_idx = if current_idx % 2 == 0 {
                current_idx + 1
            } else {
                current_idx - 1
            };

            if sibling_idx < level.len() {
                proof.push(level[sibling_idx]);
            }
            current_idx /= 2;
        }
        proof
    }

    /// Verifies that a data block is included in the tree at the given index.
    /// 
    /// # Arguments
    /// * `data` - The raw data block.
    /// * `index` - The claimed leaf index.
    /// * `proof` - The Merkle Inclusion Proof.
    /// * `root` - The expected Merkle Root.
    /// 
    /// # Returns
    /// `true` if the proof is valid, `false` otherwise.
    pub fn verify_proof(data: &[u8], index: usize, proof: &[Hash], root: &Hash) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let mut current_hash: Hash = hasher.finalize().into();

        let mut current_idx = index;

        for sibling in proof {
            let mut hasher = Sha256::new();
            if current_idx % 2 == 0 {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().into();
            current_idx /= 2;
        }

        &current_hash == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_construction_and_proof() {
        let data: Vec<&[u8]> = vec![b"record0", b"record1", b"record2", b"record3"];
        let tree = MerkleTree::from_data(&data);

        assert!(tree.root().is_some());

        let proof = tree.generate_proof(1);
        let root = tree.root().unwrap();

        assert!(MerkleTree::verify_proof(b"record1", 1, &proof, &root));
        assert!(!MerkleTree::verify_proof(b"tampered", 1, &proof, &root));
    }

    #[test]
    fn test_odd_leaf_count() {
        let data: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let tree = MerkleTree::from_data(&data);
        assert!(tree.root().is_some());
    }
}
