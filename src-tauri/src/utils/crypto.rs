use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};
use tracing::warn;

const ENCRYPTION_PREFIX: &str = "enc:v1:";

fn derive_key() -> [u8; 32] {
    let machine_id = format!(
        "{}:{}:{}:{}",
        whoami::fallible::hostname().unwrap_or_default(),
        whoami::fallible::username().unwrap_or_default(),
        format!("{:?}", whoami::platform()),
        format!("{:?}", whoami::arch()),
    );
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"arcane-codex-key-derivation-salt-v1");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt_api_key(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce = Nonce::from_slice(b"ac-kd-nonce-12");

    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ciphertext) => {
            format!("{}{}", ENCRYPTION_PREFIX, BASE64.encode(ciphertext))
        }
        Err(e) => {
            warn!("API Key 加密失败，回退到明文存储: {}", e);
            plaintext.to_string()
        }
    }
}

pub fn decrypt_api_key(ciphertext: &str) -> String {
    if ciphertext.is_empty() {
        return String::new();
    }
    if !ciphertext.starts_with(ENCRYPTION_PREFIX) {
        return ciphertext.to_string();
    }

    let encoded = &ciphertext[ENCRYPTION_PREFIX.len()..];
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce = Nonce::from_slice(b"ac-kd-nonce-12");

    match BASE64.decode(encoded) {
        Ok(data) => match cipher.decrypt(nonce, data.as_ref()) {
            Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|e| {
                warn!("API Key 解密 UTF-8 转换失败: {}", e);
                ciphertext.to_string()
            }),
            Err(e) => {
                warn!("API Key 解密失败，可能密钥已变更: {}", e);
                ciphertext.to_string()
            }
        },
        Err(e) => {
            warn!("API Key Base64 解码失败: {}", e);
            ciphertext.to_string()
        }
    }
}

#[allow(dead_code)]
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTION_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-1234567890abcdef";
        let encrypted = encrypt_api_key(original);
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX));
        assert_ne!(encrypted, original);

        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let encrypted = encrypt_api_key("");
        assert!(encrypted.is_empty());
        let decrypted = decrypt_api_key("");
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_plaintext_fallback() {
        let plaintext = "my-plain-api-key";
        let decrypted = decrypt_api_key(plaintext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("enc:v1:somebase64data"));
        assert!(!is_encrypted("plain-api-key"));
        assert!(!is_encrypted(""));
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let original = "same-key-value";
        let enc1 = encrypt_api_key(original);
        let enc2 = encrypt_api_key(original);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_encrypt_special_characters() {
        let original = "sk-proj-abc+def/ghi=jkl==";
        let encrypted = encrypt_api_key(original);
        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(decrypted, original);
    }
}
