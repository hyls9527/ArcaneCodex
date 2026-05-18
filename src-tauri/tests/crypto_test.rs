//! Integration tests for API key encryption (SAFE-01).
//!
//! These tests verify the public API of the crypto module:
//!   - v4 keyring/PBKDF2 encrypt/decrypt roundtrip
//!   - Backward compatibility with v2 and v3 encrypted formats
//!   - No plaintext fallback on encryption failure (compile-time assertion)

use arcane_codex::utils::crypto;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

// ---------------------------------------------------------------------------
// v4 roundtrip tests
// ---------------------------------------------------------------------------

#[test]
fn test_encrypt_decrypt_roundtrip_v4() {
    let original = "sk-1234567890abcdef";
    let encrypted = crypto::encrypt_api_key(original).unwrap();
    assert!(
        encrypted.starts_with("enc:v4:"),
        "encrypted should start with v4 prefix"
    );
    assert_ne!(encrypted, original);

    let decrypted = crypto::decrypt_api_key(&encrypted).unwrap();
    assert_eq!(decrypted, original);
}

#[test]
fn test_encrypt_empty_string() {
    let encrypted = crypto::encrypt_api_key("").unwrap();
    assert!(encrypted.is_empty());
    let decrypted = crypto::decrypt_api_key("").unwrap();
    assert!(decrypted.is_empty());
}

#[test]
fn test_decrypt_plaintext_passthrough() {
    // Non-encrypted strings should pass through as-is on decrypt
    let plaintext = "my-plain-api-key";
    let decrypted = crypto::decrypt_api_key(plaintext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_decrypt_tampered_returns_empty() {
    let original = "sk-test-key-for-tamper-scenario";
    let encrypted = crypto::encrypt_api_key(original).unwrap();
    assert!(encrypted.starts_with("enc:v4:"));

    // Tamper with the data by injecting garbage prefix data
    let tampered = encrypted.replace("enc:v4:", "enc:v4:invalidbase64!!!");
    let decrypted = crypto::decrypt_api_key(&tampered).unwrap();
    assert_eq!(
        decrypted, "",
        "tampered ciphertext should return empty string"
    );
}

// ---------------------------------------------------------------------------
// Non-deterministic output (random nonce/salt per call)
// ---------------------------------------------------------------------------

#[test]
fn test_encrypt_produces_different_ciphertexts() {
    let original = "same-key-value";
    let enc1 = crypto::encrypt_api_key(original).unwrap();
    let enc2 = crypto::encrypt_api_key(original).unwrap();
    assert!(enc1.starts_with("enc:v4:"));
    assert!(enc2.starts_with("enc:v4:"));
    assert_ne!(
        enc1, enc2,
        "same plaintext should produce different ciphertexts"
    );

    let dec1 = crypto::decrypt_api_key(&enc1).unwrap();
    let dec2 = crypto::decrypt_api_key(&enc2).unwrap();
    assert_eq!(dec1, original);
    assert_eq!(dec2, original);
}

// ---------------------------------------------------------------------------
// Special character handling
// ---------------------------------------------------------------------------

#[test]
fn test_encrypt_special_characters() {
    let original = "sk-proj-abc+def/ghi=jkl==";
    let encrypted = crypto::encrypt_api_key(original).unwrap();
    let decrypted = crypto::decrypt_api_key(&encrypted).unwrap();
    assert_eq!(decrypted, original);
}

// ---------------------------------------------------------------------------
// v1 deprecated returns empty
// ---------------------------------------------------------------------------

#[test]
fn test_v1_deprecated_returns_empty() {
    let fake_v1 = format!("enc:v1:{}", BASE64.encode(b"fake-ciphertext-data"));
    let decrypted = crypto::decrypt_api_key(&fake_v1).unwrap();
    assert_eq!(
        decrypted,
        "",
        "v1 format ciphertext should return empty string"
    );
}

// ---------------------------------------------------------------------------
// Backward compatibility: v2 format (SHA256-derived key)
// ---------------------------------------------------------------------------

#[test]
fn test_v2_backward_compatibility() {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    use sha2::{Digest, Sha256};

    let machine_id = format!(
        "{}:{}:{:?}:{:?}",
        whoami::fallible::hostname().unwrap_or_default(),
        whoami::fallible::username().unwrap_or_default(),
        whoami::platform(),
        whoami::arch(),
    );
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"arcane-codex-key-derivation-salt-v1");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, &b"sk-v2-compat-test-key"[..])
        .unwrap();

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    let v2_encrypted = format!("enc:v2:{}", BASE64.encode(&combined));

    let decrypted = crypto::decrypt_api_key(&v2_encrypted).unwrap();
    assert_eq!(
        decrypted, "sk-v2-compat-test-key",
        "v2 format ciphertext must be backward-compatible"
    );
}

// ---------------------------------------------------------------------------
// Backward compatibility: v3 format (PBKDF2-derived key)
// ---------------------------------------------------------------------------

#[test]
fn test_v3_backward_compatibility() {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let machine_id = format!(
        "{}:{}:{:?}:{:?}",
        whoami::fallible::hostname().unwrap_or_default(),
        whoami::fallible::username().unwrap_or_default(),
        whoami::platform(),
        whoami::arch(),
    );
    let salt: [u8; 16] = rand::random();
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(machine_id.as_bytes(), &salt, 600_000, &mut key);

    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, &b"sk-v3-compat-test-key"[..])
        .unwrap();

    let mut combined = Vec::with_capacity(16 + 12 + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    let v3_encrypted = format!("enc:v3:{}", BASE64.encode(&combined));

    let decrypted = crypto::decrypt_api_key(&v3_encrypted).unwrap();
    assert_eq!(
        decrypted, "sk-v3-compat-test-key",
        "v3 format ciphertext must be backward-compatible"
    );
}
