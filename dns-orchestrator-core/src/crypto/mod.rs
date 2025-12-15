//! 加密模块
//!
//! 提供 AES-256-GCM 加密/解密功能，用于账户导入导出的加密保护。

mod versions;

pub use versions::{get_current_iterations, get_pbkdf2_iterations, CURRENT_FILE_VERSION};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac_array;
use rand::RngCore;
use sha2::Sha256;

use crate::error::{CoreError, CoreResult};

// 从版本管理自动获取当前版本的迭代次数
const PBKDF2_ITERATIONS: u32 = get_current_iterations();
const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32; // AES-256

/// 从密码派生加密密钥（支持自定义迭代次数）
fn derive_key_with_iterations(password: &str, salt: &[u8], iterations: u32) -> [u8; KEY_LENGTH] {
    pbkdf2_hmac_array::<Sha256, KEY_LENGTH>(password.as_bytes(), salt, iterations)
}

/// 从密码派生加密密钥（使用默认迭代次数）
fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    derive_key_with_iterations(password, salt, PBKDF2_ITERATIONS)
}

/// 加密数据
///
/// # Arguments
/// * `plaintext` - 要加密的明文数据
/// * `password` - 加密密码
///
/// # Returns
/// 返回 (`salt_base64`, `nonce_base64`, `ciphertext_base64`) 元组
pub fn encrypt(plaintext: &[u8], password: &str) -> CoreResult<(String, String, String)> {
    // 生成随机盐和 nonce
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce_bytes);

    // 派生密钥
    let key = derive_key(password, &salt);

    // 创建加密器
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create cipher: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::SerializationError(format!("Encryption failed: {e}")))?;

    Ok((
        BASE64.encode(salt),
        BASE64.encode(nonce_bytes),
        BASE64.encode(ciphertext),
    ))
}

/// 解密数据
///
/// # Arguments
/// * `ciphertext_b64` - Base64 编码的密文
/// * `password` - 解密密码
/// * `salt_b64` - Base64 编码的盐值
/// * `nonce_b64` - Base64 编码的 nonce
///
/// # Returns
/// 返回解密后的明文数据
pub fn decrypt(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
) -> CoreResult<Vec<u8>> {
    decrypt_with_iterations(
        ciphertext_b64,
        password,
        salt_b64,
        nonce_b64,
        PBKDF2_ITERATIONS,
    )
}

/// 使用自定义迭代次数解密数据（用于向后兼容）
///
/// # Arguments
/// * `ciphertext_b64` - Base64 编码的密文
/// * `password` - 解密密码
/// * `salt_b64` - Base64 编码的盐值
/// * `nonce_b64` - Base64 编码的 nonce
/// * `iterations` - PBKDF2 迭代次数
///
/// # Returns
/// 返回解密后的明文数据
pub fn decrypt_with_iterations(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
    iterations: u32,
) -> CoreResult<Vec<u8>> {
    // 解码 Base64
    let salt = BASE64
        .decode(salt_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid salt: {e}")))?;
    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid nonce: {e}")))?;
    let ciphertext = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid ciphertext: {e}")))?;

    // 使用指定迭代次数派生密钥
    let key = derive_key_with_iterations(password, &salt, iterations);

    // 创建解密器
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create cipher: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 解密
    cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        CoreError::SerializationError(
            "Decryption failed: invalid password or corrupted data".to_string(),
        )
    })
}
