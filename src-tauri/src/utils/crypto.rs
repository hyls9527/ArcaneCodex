#![allow(missing_docs)]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::{Digest, Sha256};
use tracing::warn;

const ENCRYPTION_PREFIX_V1: &str = "enc:v1:";
const ENCRYPTION_PREFIX_V2: &str = "enc:v2:";
const ENCRYPTION_PREFIX_V3: &str = "enc:v3:";

/// PBKDF2 迭代次数，遵循 OWASP 2023 推荐
const PBKDF2_ITERATIONS: u32 = 600_000;

/// PBKDF2 salt 长度（16 字节 = 128 位）
const SALT_LEN: usize = 16;

/// 使用单次 SHA256 派生密钥（v2 格式，向后兼容解密用）
fn derive_key_v2() -> [u8; 32] {
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

/// 使用 PBKDF2-HMAC-SHA256 派生密钥（v3 格式）
fn derive_key_v3(salt: &[u8]) -> [u8; 32] {
    let machine_id = format!(
        "{}:{}:{:?}:{:?}",
        whoami::fallible::hostname().unwrap_or_default(),
        whoami::fallible::username().unwrap_or_default(),
        whoami::platform(),
        whoami::arch(),
    );
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(machine_id.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// 生成加密安全的随机 salt
fn generate_salt() -> [u8; SALT_LEN] {
    rand::random()
}

pub fn encrypt_api_key(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }

    // v3: PBKDF2-HMAC-SHA256 + 随机 salt
    let salt = generate_salt();
    let key = derive_key_v3(&salt);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ciphertext) => {
            // v3 格式: enc:v3: + base64(salt[16] + nonce[12] + ciphertext)
            let mut combined = Vec::with_capacity(SALT_LEN + 12 + ciphertext.len());
            combined.extend_from_slice(&salt);
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);
            format!("{}{}", ENCRYPTION_PREFIX_V3, BASE64.encode(&combined))
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

    if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V3) {
        // v3: PBKDF2-HMAC-SHA256 派生密钥，salt 嵌入密文
        match BASE64.decode(encoded) {
            Ok(data) if data.len() > SALT_LEN + 12 => {
                let salt = &data[..SALT_LEN];
                let nonce = Nonce::from_slice(&data[SALT_LEN..SALT_LEN + 12]);
                let ciphertext_only = &data[SALT_LEN + 12..];

                let key = derive_key_v3(salt);
                let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");

                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key v3 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    }),
                    Err(e) => {
                        warn!("API Key v3 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        String::new()
                    }
                }
            }
            _ => {
                warn!("API Key v3 格式无效，已返回空值");
                String::new()
            }
        }
    } else if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V2) {
        // v2: 单次 SHA256 派生密钥（向后兼容）
        let key = derive_key_v2();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");

        match BASE64.decode(encoded) {
            Ok(data) if data.len() > 12 => {
                let nonce = Nonce::from_slice(&data[..12]);
                let ciphertext_only = &data[12..];
                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key v2 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    }),
                    Err(e) => {
                        warn!("API Key v2 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        String::new()
                    }
                }
            }
            _ => {
                warn!("API Key v2 格式无效，已返回空值");
                String::new()
            }
        }
    } else if ciphertext.starts_with(ENCRYPTION_PREFIX_V1) {
        // v1: 固定 Nonce 安全漏洞，已废弃，不允许解密
        warn!(
            "拒绝解密 v1 格式密文（固定 Nonce 安全漏洞）。请使用 v3 格式重新加密此备份。原始密文前缀: {}",
            &ciphertext[..ciphertext.len().min(20)]
        );
        String::new()
    } else {
        ciphertext.to_string()
    }
}

#[allow(dead_code)]
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTION_PREFIX_V3) || value.starts_with(ENCRYPTION_PREFIX_V2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-1234567890abcdef";
        let encrypted = encrypt_api_key(original);
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V3));
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
    fn test_decrypt_v3_with_wrong_key_returns_empty() {
        let original = "sk-test-key-for-wrong-key-scenario";
        let encrypted = encrypt_api_key(original);
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V3));

        let tampered = encrypted.replace(ENCRYPTION_PREFIX_V3, "enc:v3:invalidbase64!!!");
        let decrypted = decrypt_api_key(&tampered);
        assert_eq!(decrypted, "", "密钥变更或密文损坏时应返回空字符串");
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("enc:v3:somebase64data"));
        assert!(is_encrypted("enc:v2:somebase64data"));
        assert!(!is_encrypted("enc:v1:somebase64data")); // v1 已废弃
        assert!(!is_encrypted("plain-api-key"));
        assert!(!is_encrypted(""));
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let original = "same-key-value";
        let enc1 = encrypt_api_key(original);
        let enc2 = encrypt_api_key(original);
        assert_ne!(enc1, enc2, "随机 salt+Nonce 应使相同明文产生不同密文");

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
    fn test_v1_deprecated_returns_empty() {
        let fake_v1_encrypted = format!(
            "{}{}",
            ENCRYPTION_PREFIX_V1,
            BASE64.encode(b"fake-ciphertext-data")
        );
        let decrypted = decrypt_api_key(&fake_v1_encrypted);
        assert_eq!(decrypted, "", "v1 格式密文应返回空字符串（已废弃）");
    }

    #[test]
    fn test_v3_nonce_and_salt_uniqueness() {
        let original = "test-nonce-salt-uniqueness";
        let mut salts = std::collections::HashSet::new();
        let mut nonces = std::collections::HashSet::new();

        for _ in 0..10 {
            let encrypted = encrypt_api_key(original);
            assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V3));
            let encoded = &encrypted[ENCRYPTION_PREFIX_V3.len()..];
            let data = BASE64.decode(encoded).unwrap();
            assert!(data.len() > SALT_LEN + 12);
            let salt = &data[..SALT_LEN];
            let nonce = &data[SALT_LEN..SALT_LEN + 12];
            salts.insert(salt.to_vec());
            nonces.insert(nonce.to_vec());
        }

        assert!(salts.len() >= 9, "10 次加密应产生至少 9 个不同的 Salt");
        assert!(nonces.len() >= 9, "10 次加密应产生至少 9 个不同的 Nonce");
    }

    #[test]
    fn test_v2_backward_compatibility() {
        // 模拟 v2 格式加密：使用单次 SHA256 派生密钥 + 随机 Nonce
        let original = "sk-backward-compat-test-key";
        let key = derive_key_v2();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, original.as_bytes()).unwrap();

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        let v2_encrypted = format!("{}{}", ENCRYPTION_PREFIX_V2, BASE64.encode(&combined));

        // v2 格式密文应能正常解密
        let decrypted = decrypt_api_key(&v2_encrypted);
        assert_eq!(decrypted, original, "v2 格式密文应能向后兼容解密");
    }

    #[test]
    fn test_pbkdf2_iterations_value() {
        assert_eq!(
            PBKDF2_ITERATIONS, 600_000,
            "PBKDF2 迭代次数应符合 OWASP 2023 推荐"
        );
    }
}
