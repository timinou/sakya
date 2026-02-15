//! Property-based tests for sakya-crypto using proptest.
//!
//! These tests verify cryptographic invariants hold for arbitrary inputs.

use proptest::prelude::*;
use sakya_crypto::{
    document_key::DocumentKey,
    encryptor::{Encryptor, XChaCha20Encryptor},
    hash_chain::HashChain,
    key_exchange::EphemeralKeyPair,
    signer::DeviceKeyPair,
};

/// Strategy for generating random byte vectors of reasonable size.
fn arbitrary_bytes(max_size: usize) -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..=max_size)
}

/// Strategy for generating non-empty byte vectors.
fn non_empty_bytes(max_size: usize) -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 1..=max_size)
}

// Property 1: Encrypt-decrypt identity
// For any plaintext, key, and AAD, decrypt(encrypt(m, k, aad), k, aad) == m
proptest! {
    #[test]
    fn encrypt_decrypt_identity(
        plaintext in arbitrary_bytes(10000),
        aad in arbitrary_bytes(1000),
    ) {
        let key = DocumentKey::generate();
        let encryptor = XChaCha20Encryptor::new(*key.as_bytes());

        let envelope = encryptor
            .encrypt(&plaintext, &aad)
            .expect("encryption should succeed");

        let decrypted = encryptor
            .decrypt(&envelope)
            .expect("decryption should succeed");

        prop_assert_eq!(decrypted, plaintext);
    }
}

// Property 1b: Encrypt with different keys produces different ciphertexts
proptest! {
    #[test]
    fn different_keys_different_ciphertexts(
        plaintext in non_empty_bytes(1000),
        aad in arbitrary_bytes(100),
    ) {
        let key1 = DocumentKey::generate();
        let key2 = DocumentKey::generate();

        // Skip test if by astronomical chance we generate the same key
        prop_assume!(key1.as_bytes() != key2.as_bytes());

        let enc1 = XChaCha20Encryptor::new(*key1.as_bytes());
        let enc2 = XChaCha20Encryptor::new(*key2.as_bytes());

        let envelope1 = enc1.encrypt(&plaintext, &aad).expect("encrypt 1 should succeed");
        let envelope2 = enc2.encrypt(&plaintext, &aad).expect("encrypt 2 should succeed");

        // The ciphertexts should be different (overwhelmingly likely)
        prop_assert_ne!(envelope1.ciphertext, envelope2.ciphertext);
    }
}

// Property 1c: Wrong key fails decryption
proptest! {
    #[test]
    fn wrong_key_fails_decryption(
        plaintext in arbitrary_bytes(1000),
        aad in arbitrary_bytes(100),
    ) {
        let key1 = DocumentKey::generate();
        let key2 = DocumentKey::generate();

        // Skip test if by astronomical chance we generate the same key
        prop_assume!(key1.as_bytes() != key2.as_bytes());

        let enc1 = XChaCha20Encryptor::new(*key1.as_bytes());
        let enc2 = XChaCha20Encryptor::new(*key2.as_bytes());

        let envelope = enc1.encrypt(&plaintext, &aad).expect("encrypt should succeed");
        let result = enc2.decrypt(&envelope);

        prop_assert!(result.is_err(), "decryption with wrong key should fail");
    }
}

// Property 2: Sign-verify identity
// For any message, sign(m, sk) verifies with corresponding pk
proptest! {
    #[test]
    fn sign_verify_identity(
        message in arbitrary_bytes(10000),
    ) {
        let keypair = DeviceKeyPair::generate();
        let signed_msg = keypair.sign(&message);

        let result = signed_msg.verify();
        prop_assert!(result.is_ok(), "valid signature should verify");

        // Also verify the payload is preserved
        prop_assert_eq!(signed_msg.payload, message);
    }
}

// Property 2b: Tampered message fails verification
proptest! {
    #[test]
    fn tampered_message_fails_verification(
        message in non_empty_bytes(1000),
        tamper_index in any::<usize>(),
        tamper_value in any::<u8>(),
    ) {
        let keypair = DeviceKeyPair::generate();
        let mut signed_msg = keypair.sign(&message);

        // Tamper with a byte in the payload
        let index = tamper_index % signed_msg.payload.len();
        let original = signed_msg.payload[index];

        // Make sure we actually change the value
        let new_value = if tamper_value == original {
            original.wrapping_add(1)
        } else {
            tamper_value
        };

        signed_msg.payload[index] = new_value;

        let result = signed_msg.verify();
        prop_assert!(result.is_err(), "tampered message should fail verification");
    }
}

// Property 2c: Different messages produce different signatures
proptest! {
    #[test]
    fn different_messages_different_signatures(
        message1 in arbitrary_bytes(1000),
        message2 in arbitrary_bytes(1000),
    ) {
        prop_assume!(message1 != message2);

        let keypair = DeviceKeyPair::generate();
        let signed1 = keypair.sign(&message1);
        let signed2 = keypair.sign(&message2);

        prop_assert_ne!(
            signed1.signature.to_bytes(),
            signed2.signature.to_bytes(),
            "different messages should produce different signatures"
        );
    }
}

// Property 3: Key exchange commutativity
// dh(a, B) == dh(b, A) for any two keypairs
proptest! {
    #[test]
    fn key_exchange_commutativity(
        _seed in any::<u64>(), // Just to get multiple test runs
    ) {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let secret_alice = alice.derive_shared_secret(&bob.public_key());
        let secret_bob = bob.derive_shared_secret(&alice.public_key());

        prop_assert_eq!(
            secret_alice,
            secret_bob,
            "key exchange should be commutative"
        );
    }
}

// Property 3b: Shared secrets can be used for encryption
proptest! {
    #[test]
    fn shared_secret_encryption_works(
        plaintext in arbitrary_bytes(1000),
        aad in arbitrary_bytes(100),
    ) {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        // Alice encrypts using shared secret
        let alice_secret = alice.derive_shared_secret(&bob.public_key());
        let alice_enc = XChaCha20Encryptor::new(alice_secret);
        let envelope = alice_enc.encrypt(&plaintext, &aad)
            .expect("alice encryption should succeed");

        // Bob decrypts using shared secret
        let bob_secret = bob.derive_shared_secret(&alice.public_key());
        let bob_enc = XChaCha20Encryptor::new(bob_secret);
        let decrypted = bob_enc.decrypt(&envelope)
            .expect("bob decryption should succeed");

        prop_assert_eq!(decrypted, plaintext);
    }
}

// Property 4: Hash chain integrity
// Chain is append-only, verify(chain[0..n]) implies verify(chain[0..n-1])
proptest! {
    #[test]
    fn hash_chain_append_only(
        data_items in prop::collection::vec(
            arbitrary_bytes(1000),
            1..=20
        ),
    ) {
        let mut chain = HashChain::new();
        let genesis_proof = *chain.current_proof();

        // Build up the chain and collect all proofs
        let mut proofs = vec![genesis_proof];
        let mut entries = vec![];

        for data in &data_items {
            let proof = chain.append(data);
            proofs.push(proof);
            entries.push(data.clone());
        }

        // Verify that each step in the chain is valid
        for i in 0..entries.len() {
            let prev_proof = &proofs[i];
            let data = &entries[i];
            let expected_proof = &proofs[i + 1];

            prop_assert!(
                HashChain::verify_step(prev_proof, data, expected_proof),
                "chain step {} should verify", i
            );
        }

        // Verify that reordering breaks the chain
        if entries.len() >= 2 {
            // Try to verify entry 1 directly after genesis (skipping entry 0)
            let skip_result = HashChain::verify_step(
                &proofs[0],  // genesis
                &entries[1], // second entry
                &proofs[2],  // proof after second entry
            );
            prop_assert!(
                !skip_result,
                "skipping an entry should break verification"
            );
        }
    }
}

// Property 4b: Hash chain proofs are unique
proptest! {
    #[test]
    fn hash_chain_proofs_unique(
        data1 in arbitrary_bytes(100),
        data2 in arbitrary_bytes(100),
    ) {
        prop_assume!(data1 != data2);

        let mut chain1 = HashChain::new();
        let mut chain2 = HashChain::new();

        let proof1 = chain1.append(&data1);
        let proof2 = chain2.append(&data2);

        prop_assert_ne!(
            proof1,
            proof2,
            "different data should produce different proofs"
        );
    }
}

// Property 5: DocumentKey derivation determinism
// Same master + context always produces same key
proptest! {
    #[test]
    fn document_key_derivation_deterministic(
        master_secret in arbitrary_bytes(100),
        context in prop::string::string_regex("[a-zA-Z0-9-]{1,100}")
            .expect("valid regex"),
    ) {
        let key1 = DocumentKey::derive(&master_secret, &context);
        let key2 = DocumentKey::derive(&master_secret, &context);

        prop_assert_eq!(
            key1.as_bytes(),
            key2.as_bytes(),
            "same inputs should produce same derived key"
        );
    }
}

// Property 5b: Different contexts produce different keys
proptest! {
    #[test]
    fn different_contexts_different_keys(
        master_secret in arbitrary_bytes(100),
        context1 in prop::string::string_regex("[a-zA-Z0-9-]{1,100}")
            .expect("valid regex"),
        context2 in prop::string::string_regex("[a-zA-Z0-9-]{1,100}")
            .expect("valid regex"),
    ) {
        prop_assume!(context1 != context2);

        let key1 = DocumentKey::derive(&master_secret, &context1);
        let key2 = DocumentKey::derive(&master_secret, &context2);

        prop_assert_ne!(
            key1.as_bytes(),
            key2.as_bytes(),
            "different contexts should produce different keys"
        );
    }
}

// Property 5c: Different master secrets produce different keys
proptest! {
    #[test]
    fn different_masters_different_keys(
        master1 in non_empty_bytes(100),
        master2 in non_empty_bytes(100),
        context in prop::string::string_regex("[a-zA-Z0-9-]{1,100}")
            .expect("valid regex"),
    ) {
        prop_assume!(master1 != master2);

        let key1 = DocumentKey::derive(&master1, &context);
        let key2 = DocumentKey::derive(&master2, &context);

        prop_assert_ne!(
            key1.as_bytes(),
            key2.as_bytes(),
            "different master secrets should produce different keys"
        );
    }
}

// Property 5d: Derived keys work with encryption
proptest! {
    #[test]
    fn derived_keys_work_with_encryption(
        master in arbitrary_bytes(100),
        context in prop::string::string_regex("[a-zA-Z0-9-]{1,100}")
            .expect("valid regex"),
        plaintext in arbitrary_bytes(1000),
        aad in arbitrary_bytes(100),
    ) {
        let key = DocumentKey::derive(&master, &context);
        let enc = XChaCha20Encryptor::new(*key.as_bytes());

        let envelope = enc.encrypt(&plaintext, &aad)
            .expect("encryption should succeed");
        let decrypted = enc.decrypt(&envelope)
            .expect("decryption should succeed");

        prop_assert_eq!(decrypted, plaintext);
    }
}
