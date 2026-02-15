//! BLAKE2b-256 hash chain for tamper-evident operation logging.
//!
//! Provides [`HashChain`] for maintaining an append-only chain of proofs.
//! Each proof is derived from the previous proof concatenated with new data,
//! enabling detection of dropped or reordered updates.

use blake2::digest::{consts::U32, Digest};

/// BLAKE2b with 32-byte (256-bit) output.
type Blake2b256 = blake2::Blake2b<U32>;

/// A BLAKE2b-256 hash chain for integrity verification.
///
/// The chain starts from a deterministic genesis proof and each subsequent
/// entry is `BLAKE2b-256(prev_proof || data)`.
pub struct HashChain {
    current_proof: [u8; 32],
}

impl HashChain {
    /// Create a new hash chain with the genesis proof.
    ///
    /// The genesis proof is `BLAKE2b-256("sakya-genesis")`.
    pub fn new() -> Self {
        let mut hasher = Blake2b256::new();
        hasher.update(b"sakya-genesis");
        let result = hasher.finalize();
        let mut proof = [0u8; 32];
        proof.copy_from_slice(&result);
        Self {
            current_proof: proof,
        }
    }

    /// Append data to the chain, returning the new proof.
    ///
    /// The new proof is `BLAKE2b-256(current_proof || data)`.
    pub fn append(&mut self, data: &[u8]) -> [u8; 32] {
        let proof = Self::compute_proof(&self.current_proof, data);
        self.current_proof = proof;
        proof
    }

    /// Get the current proof (head of the chain).
    pub fn current_proof(&self) -> &[u8; 32] {
        &self.current_proof
    }

    /// Verify a single step in the chain.
    ///
    /// Returns `true` if `BLAKE2b-256(prev_proof || data) == expected_proof`.
    pub fn verify_step(prev_proof: &[u8; 32], data: &[u8], expected_proof: &[u8; 32]) -> bool {
        let computed = Self::compute_proof(prev_proof, data);
        computed == *expected_proof
    }

    /// Compute `BLAKE2b-256(prev_proof || data)`.
    fn compute_proof(prev_proof: &[u8; 32], data: &[u8]) -> [u8; 32] {
        let mut hasher = Blake2b256::new();
        hasher.update(prev_proof);
        hasher.update(data);
        let result = hasher.finalize();
        let mut proof = [0u8; 32];
        proof.copy_from_slice(&result);
        proof
    }
}

impl Default for HashChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_proof_is_deterministic() {
        let chain1 = HashChain::new();
        let chain2 = HashChain::new();

        assert_eq!(
            chain1.current_proof(),
            chain2.current_proof(),
            "genesis proofs should be identical"
        );
    }

    #[test]
    fn append_produces_new_proof() {
        let mut chain = HashChain::new();
        let genesis = *chain.current_proof();

        let proof = chain.append(b"first entry");

        assert_ne!(
            genesis, proof,
            "appending data should produce a different proof"
        );
        assert_eq!(
            chain.current_proof(),
            &proof,
            "current_proof should match the returned proof"
        );
    }

    #[test]
    fn verify_step_succeeds_for_valid_data() {
        let mut chain = HashChain::new();
        let prev = *chain.current_proof();
        let data = b"entry data";

        let proof = chain.append(data);

        assert!(
            HashChain::verify_step(&prev, data, &proof),
            "verify_step should succeed for valid data"
        );
    }

    #[test]
    fn verify_step_fails_for_tampered_data() {
        let mut chain = HashChain::new();
        let prev = *chain.current_proof();

        let proof = chain.append(b"original data");

        assert!(
            !HashChain::verify_step(&prev, b"tampered data", &proof),
            "verify_step should fail for tampered data"
        );
    }

    #[test]
    fn chain_of_three_entries_verifies() {
        let mut chain = HashChain::new();

        let proof0 = *chain.current_proof();
        let data1 = b"entry 1";
        let proof1 = chain.append(data1);

        let data2 = b"entry 2";
        let proof2 = chain.append(data2);

        let data3 = b"entry 3";
        let proof3 = chain.append(data3);

        // Verify the entire chain step by step
        assert!(
            HashChain::verify_step(&proof0, data1, &proof1),
            "step 1 should verify"
        );
        assert!(
            HashChain::verify_step(&proof1, data2, &proof2),
            "step 2 should verify"
        );
        assert!(
            HashChain::verify_step(&proof2, data3, &proof3),
            "step 3 should verify"
        );

        // Verify that skipping a step fails
        assert!(
            !HashChain::verify_step(&proof0, data2, &proof2),
            "skipping step 1 should fail verification"
        );
    }
}
