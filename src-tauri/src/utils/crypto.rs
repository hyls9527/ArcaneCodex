#![allow(missing_docs)]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};
use tracing::warn;

const ENCRYPTION_PREFIX_V1: &str = "enc:v1:";
const ENCRYPTION_PREFIX_V2: &str = "enc:v2:";

fn derive_key() -> [u8; 32] {
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
    key
}

pub fn encrypt_api_key(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ciphertext) => {
            let mut combined = Vec::with_capacity(12 + ciphertext.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);
            format!("{}{}", ENCRYPTION_PREFIX_V2, BASE64.encode(&combined))
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

    if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V2) {
        let key = derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");

        match BASE64.decode(encoded) {
            Ok(data) if data.len() > 12 => {
                let nonce = Nonce::from_slice(&data[..12]);
                let ciphertext_only = &data[12..];
                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    }),
                    Err(e) => {
                        warn!("API Key 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        String::new()
                    }
                }
            }
            _ => {
                warn!("API Key v2 格式无效，已返回空值");
                String::new()
            }
        }
    } else if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V1) {
        let key = derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
        let nonce = Nonce::from_slice(b"ac-kd-nonce-12");

        match BASE64.decode(encoded) {
            Ok(data) => match cipher.decrypt(nonce, data.as_ref()) {
                Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|e| {
                    warn!("API Key v1 解密 UTF-8 转换失败，已返回空值: {}", e);
                    String::new()
                }),
                Err(e) => {
                    warn!("API Key v1 解密失败，已返回空值，请重新配置 API Key: {}", e);
                    String::new()
                }
            },
            Err(e) => {
                warn!("API Key v1 Base64 解码失败，已返回空值: {}", e);
                String::new()
            }
        }
    } else {
        ciphertext.to_string()
    }
}

#[allow(dead_code)]
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTION_PREFIX_V1) || value.starts_with(ENCRYPTION_PREFIX_V2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-1234567890abcdef";
        let encrypted = encrypt_api_key(original);
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V2));
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
    fn test_decrypt_v2_with_wrong_key_returns_empty() {
        let original = "sk-test-key-for-wrong-key-scenario";
        let encrypted = encrypt_api_key(original);
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V2));

        let tampered = encrypted.replace(ENCRYPTION_PREFIX_V2, "enc:v2:invalidbase64!!!");
        let decrypted = decrypt_api_key(&tampered);
        assert_eq!(decrypted, "", "密钥变更或密文损坏时应返回空字符串");
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("enc:v1:somebase64data"));
        assert!(is_encrypted("enc:v2:somebase64data"));
        assert!(!is_encrypted("plain-api-key"));
        assert!(!is_encrypted(""));
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let original = "same-key-value";
        let enc1 = encrypt_api_key(original);
        let enc2 = encrypt_api_key(original);
        assert_ne!(enc1, enc2, "随机 Nonce 应使相同明文产生不同密文");

        let dec1 = decrypt_api_key(&enc1);
        let dec2 = decrypt_api_key(&enc2);
        assert_eq!(dec1, original);
        assert_eq!(dec2, original);
    }

    #[test]
    fn test_encrypt_special_characters() {
        let original = "sk-proj-abc+def/ghi=jkl==";
        let encrypted = encrypt_api_key(original);
        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_v1_backward_compatibility() {
        let key = derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
        let nonce = Nonce::from_slice(b"ac-kd-nonce-12");
        let original = "legacy-api-key-value";
        let ciphertext = cipher.encrypt(nonce, original.as_bytes()).unwrap();
        let v1_encrypted = format!("{}{}", ENCRYPTION_PREFIX_V1, BASE64.encode(&ciphertext));

        let decrypted = decrypt_api_key(&v1_encrypted);
        assert_eq!(decrypted, original, "v1 加密的旧密文应能正确解密");
    }

    #[test]
    fn test_v2_nonce_uniqueness() {
        let original = "test-nonce-uniqueness";
        let mut nonces = std::collections::HashSet::new();

        for _ in 0..10 {
            let encrypted = encrypt_api_key(original);
            assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V2));
            let encoded = &encrypted[ENCRYPTION_PREFIX_V2.len()..];
            let data = BASE64.decode(encoded).unwrap();
            assert!(data.len() > 12);
            let nonce_bytes = &data[..12];
            nonces.insert(nonce_bytes.to_vec());
        }

        assert!(
            nonces.len() >= 9,
            "10 次加密应产生至少 9 个不同的 Nonce（极低碰撞概率）"
        );
    }
}
