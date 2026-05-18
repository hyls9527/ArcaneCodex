//! API key encryption/decryption using AES-256-GCM with OS keyring or PBKDF2 key derivation.
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::{Digest, Sha256};
use crate::utils::error::{AppError, AppResult};
use tracing::{info, warn};

/// Version 1 encryption prefix format (legacy).
const ENCRYPTION_PREFIX_V1: &str = "enc:v1:";
/// Version 2 encryption prefix format using single SHA256 key derivation.
const ENCRYPTION_PREFIX_V2: &str = "enc:v2:";
/// Version 3 encryption prefix format using PBKDF2-HMAC-SHA256 key derivation.
const ENCRYPTION_PREFIX_V3: &str = "enc:v3:";
/// Version 4 encryption prefix format using OS keyring master key or PBKDF2 fallback.
const ENCRYPTION_PREFIX_V4: &str = "enc:v4:";

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

/// 从操作系统密钥环获取或创建 32 字节主密钥
fn get_keyring_master_key() -> Result<[u8; 32], String> {
    let entry = keyring::Entry::new("arcane-codex", "encryption-master-key")
        .map_err(|e| format!("failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => {
            let decoded = BASE64.decode(password.as_bytes())
                .map_err(|e| format!("failed to decode keyring master key: {}", e))?;
            if decoded.len() != 32 {
                return Err(format!(
                    "keyring master key has unexpected length: {}",
                    decoded.len()
                ));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&decoded);
            info!("Retrieved encryption master key from OS keyring");
            Ok(key)
        }
        Err(keyring::Error::NoEntry) => {
            // Key doesn't exist yet — generate a new 32-byte master key and store it
            let key: [u8; 32] = rand::random();
            let encoded = BASE64.encode(key);
            entry
                .set_password(&encoded)
                .map_err(|e| format!("failed to store master key in keyring: {}", e))?;
            info!("Generated and stored new encryption master key in OS keyring");
            Ok(key)
        }
        Err(e) => {
            // Keyring unavailable (headless/CI, or platform not supported)
            Err(format!("keyring unavailable: {}", e))
        }
    }
}

/// Encrypts an API key using AES-256-GCM. Uses the OS keyring master key if available, otherwise falls back to PBKDF2-HMAC-SHA256 key derivation from machine identity. Returns the encrypted key string with v4 format prefix.
pub fn encrypt_api_key(plaintext: &str) -> AppResult<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    // 优先 OS 密钥环，失败则回退 PBKDF2（v4 格式）
    let (key, salt) = match get_keyring_master_key() {
        Ok(master_key) => {
            // Keyring 路径：salt 为全零（不用作派生，仅占位）
            info!("Using OS keyring for API key encryption");
            (master_key, [0u8; SALT_LEN])
        }
        Err(e) => {
            // PBKDF2 回退路径：随机 salt + derive_key_v3 派生
            warn!("Keyring unavailable for encryption, using PBKDF2: {}", e);
            let salt = generate_salt();
            let key = derive_key_v3(&salt);
            (key, salt)
        }
    };

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::config(format!("AES-256-GCM init failed: {}", e)))?;

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::config(format!("Encryption failed: {}", e)))?;

    // v4 格式: enc:v4: + base64(salt[16] + nonce[12] + ciphertext)
    let mut combined = Vec::with_capacity(SALT_LEN + 12 + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(format!("{}{}", ENCRYPTION_PREFIX_V4, BASE64.encode(&combined)))
}

/// Decrypts an API key previously encrypted with encrypt_api_key. Supports v4 (keyring or PBKDF2), v3 (PBKDF2), and v2 (SHA256) formats. Falls back to returning the input as-is for plaintext keys or empty strings.
pub fn decrypt_api_key(ciphertext: &str) -> AppResult<String> {
    if ciphertext.is_empty() {
        return Ok(String::new());
    }

    if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V4) {
        // v4: keyring 或 PBKDF2 回退
        match BASE64.decode(encoded) {
            Ok(data) if data.len() > SALT_LEN + 12 => {
                let salt = &data[..SALT_LEN];
                let nonce = Nonce::from_slice(&data[SALT_LEN..SALT_LEN + 12]);
                let ciphertext_only = &data[SALT_LEN + 12..];

                let key = if salt.iter().all(|&b| b == 0) {
                    // Keyring 路径：salt 全零，密钥从 keyring 获取
                    get_keyring_master_key().map_err(|e| {
                        AppError::config(format!("Keyring unavailable for decryption: {}", e))
                    })?
                } else {
                    // PBKDF2 回退路径：salt 为真实随机值，从 machine_id 派生
                    derive_key_v3(salt)
                };

                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| AppError::config(format!("AES-256-GCM init failed: {}", e)))?;

                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => Ok(String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key v4 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    })),
                    Err(e) => {
                        warn!("API Key v4 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        Ok(String::new())
                    }
                }
            }
            _ => {
                warn!("API Key v4 格式无效，已返回空值");
                Ok(String::new())
            }
        }
    } else if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V3) {
        // v3: PBKDF2-HMAC-SHA256 派生密钥，salt 嵌入密文
        match BASE64.decode(encoded) {
            Ok(data) if data.len() > SALT_LEN + 12 => {
                let salt = &data[..SALT_LEN];
                let nonce = Nonce::from_slice(&data[SALT_LEN..SALT_LEN + 12]);
                let ciphertext_only = &data[SALT_LEN + 12..];

                let key = derive_key_v3(salt);
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| AppError::config(format!("AES-256-GCM init failed: {}", e)))?;

                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => Ok(String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key v3 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    })),
                    Err(e) => {
                        warn!("API Key v3 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        Ok(String::new())
                    }
                }
            }
            _ => {
                warn!("API Key v3 格式无效，已返回空值");
                Ok(String::new())
            }
        }
    } else if let Some(encoded) = ciphertext.strip_prefix(ENCRYPTION_PREFIX_V2) {
        // v2: 单次 SHA256 派生密钥（向后兼容）
        let key = derive_key_v2();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| AppError::config(format!("AES-256-GCM init failed: {}", e)))?;

        match BASE64.decode(encoded) {
            Ok(data) if data.len() > 12 => {
                let nonce = Nonce::from_slice(&data[..12]);
                let ciphertext_only = &data[12..];
                match cipher.decrypt(nonce, ciphertext_only) {
                    Ok(plaintext) => Ok(String::from_utf8(plaintext).unwrap_or_else(|e| {
                        warn!("API Key v2 解密 UTF-8 转换失败，已返回空值: {}", e);
                        String::new()
                    })),
                    Err(e) => {
                        warn!("API Key v2 解密失败，已返回空值，请重新配置 API Key: {}", e);
                        Ok(String::new())
                    }
                }
            }
            _ => {
                warn!("API Key v2 格式无效，已返回空值");
                Ok(String::new())
            }
        }
    } else if ciphertext.starts_with(ENCRYPTION_PREFIX_V1) {
        // v1: 固定 Nonce 安全漏洞，已废弃，不允许解密
        warn!(
            "拒绝解密 v1 格式密文（固定 Nonce 安全漏洞）。请使用 v3 格式重新加密此备份。原始密文前缀: {}",
            &ciphertext[..ciphertext.len().min(20)]
        );
        Ok(String::new())
    } else {
        Ok(ciphertext.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-1234567890abcdef";
        let encrypted = encrypt_api_key(original).unwrap();
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V4));
        assert_ne!(encrypted, original);

        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let encrypted = encrypt_api_key("").unwrap();
        assert!(encrypted.is_empty());
        let decrypted = decrypt_api_key("").unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_plaintext_fallback() {
        let plaintext = "my-plain-api-key";
        let decrypted = decrypt_api_key(plaintext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_v3_with_wrong_key_returns_empty() {
        let original = "sk-test-key-for-wrong-key-scenario";
        let encrypted = encrypt_api_key(original).unwrap();
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V4));

        let tampered = encrypted.replace(ENCRYPTION_PREFIX_V4, "enc:v4:invalidbase64!!!");
        let decrypted = decrypt_api_key(&tampered).unwrap();
        assert_eq!(decrypted, "", "密钥变更或密文损坏时应返回空字符串");
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let original = "same-key-value";
        let enc1 = encrypt_api_key(original).unwrap();
        let enc2 = encrypt_api_key(original).unwrap();
        assert!(enc1.starts_with(ENCRYPTION_PREFIX_V4));
        assert!(enc2.starts_with(ENCRYPTION_PREFIX_V4));
        assert_ne!(enc1, enc2, "随机 salt+Nonce 应使相同明文产生不同密文");

        let dec1 = decrypt_api_key(&enc1).unwrap();
        let dec2 = decrypt_api_key(&enc2).unwrap();
        assert_eq!(dec1, original);
        assert_eq!(dec2, original);
    }

    #[test]
    fn test_encrypt_special_characters() {
        let original = "sk-proj-abc+def/ghi=jkl==";
        let encrypted = encrypt_api_key(original).unwrap();
        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_v1_deprecated_returns_empty() {
        let fake_v1_encrypted = format!(
            "{}{}",
            ENCRYPTION_PREFIX_V1,
            BASE64.encode(b"fake-ciphertext-data")
        );
        let decrypted = decrypt_api_key(&fake_v1_encrypted).unwrap();
        assert_eq!(decrypted, "", "v1 格式密文应返回空字符串（已废弃）");
    }

    #[test]
    fn test_v4_nonce_and_salt_uniqueness() {
        let original = "test-nonce-salt-uniqueness";
        let mut salts = std::collections::HashSet::new();
        let mut nonces = std::collections::HashSet::new();

        for _ in 0..10 {
            let encrypted = encrypt_api_key(original).unwrap();
            assert!(encrypted.starts_with(ENCRYPTION_PREFIX_V4));
            let encoded = &encrypted[ENCRYPTION_PREFIX_V4.len()..];
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
        let decrypted = decrypt_api_key(&v2_encrypted).unwrap();
        assert_eq!(decrypted, original, "v2 格式密文应能向后兼容解密");
    }

    #[test]
    fn test_no_plaintext_fallback_in_encrypt() {
        // Code review assertion: verify the encrypt_api_key function
        // does not have a plaintext fallback branch
        let source = include_str!("crypto.rs");
        // The plaintext fallback used the string "回退到明文存储"
        assert!(
            !source.contains("回退到明文存储"),
            "encrypt_api_key must not fall back to plaintext storage"
        );
    }

    #[test]
    fn test_pbkdf2_iterations_value() {
        assert_eq!(
            PBKDF2_ITERATIONS, 600_000,
            "PBKDF2 迭代次数应符合 OWASP 2023 推荐"
        );
    }
}
