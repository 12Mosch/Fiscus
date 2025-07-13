use crate::error::FiscusError;
/// Core types and data structures for the encryption service
///
/// This module defines all the fundamental types used throughout the encryption
/// system, including encrypted data containers, keys, metadata, and result types.
///
/// ## Security Note on Enum Zeroization
///
/// Enum types in this module (EncryptionAlgorithm, KeyType, etc.) do NOT implement
/// the Zeroize trait. This is intentional - enum discriminants cannot be properly
/// zeroized in memory, and providing a misleading implementation that just changes
/// the enum value would give a false sense of security. When these enums are used
/// in security-sensitive contexts (like EncryptionKey), they are marked with
/// #[zeroize(skip)] to prevent automatic zeroization attempts.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Result type for encryption operations
pub type EncryptionResult<T> = Result<T, FiscusError>;

/// Supported encryption algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    /// AES-256 in Galois/Counter Mode (authenticated encryption)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (authenticated encryption)
    ChaCha20Poly1305,
    /// RSA with 4096-bit keys
    Rsa4096,
    /// Ed25519 elliptic curve cryptography
    Ed25519,
    /// X25519 key exchange
    X25519,
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::Aes256Gcm => write!(f, "aes256_gcm"),
            EncryptionAlgorithm::ChaCha20Poly1305 => write!(f, "chacha20_poly1305"),
            EncryptionAlgorithm::Rsa4096 => write!(f, "rsa4096"),
            EncryptionAlgorithm::Ed25519 => write!(f, "ed25519"),
            EncryptionAlgorithm::X25519 => write!(f, "x25519"),
        }
    }
}

/// Types of encryption keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyType {
    /// Symmetric encryption key
    Symmetric,
    /// Asymmetric public key
    PublicKey,
    /// Asymmetric private key
    PrivateKey,
    /// Key derivation key
    DerivationKey,
    /// Master key for key encryption
    MasterKey,
}

/// Key derivation algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyDerivationAlgorithm {
    /// Argon2id (recommended for password hashing)
    Argon2id,
    /// PBKDF2 with SHA-256
    Pbkdf2Sha256,
    /// Scrypt
    Scrypt,
    /// HKDF with SHA-256
    HkdfSha256,
}

/// Container for encrypted data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// Initialization vector or nonce
    pub nonce: Vec<u8>,
    /// Authentication tag (for authenticated encryption)
    pub tag: Option<Vec<u8>>,
    /// Metadata about the encryption
    pub metadata: EncryptionMetadata,
}

/// Metadata associated with encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Algorithm used for encryption
    pub algorithm: EncryptionAlgorithm,
    /// Key identifier or derivation info
    pub key_id: String,
    /// Timestamp when encrypted
    pub encrypted_at: DateTime<Utc>,
    /// Version of the encryption scheme
    pub version: u32,
    /// Additional authenticated data (AAD)
    pub aad: Option<Vec<u8>>,
    /// Salt used for key derivation (if applicable)
    pub salt: Option<Vec<u8>>,
}

/// Secure container for encryption keys
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct EncryptionKey {
    /// The key material
    pub key_data: SecureBytes,
    /// Type of key
    /// Note: Enum fields are skipped from zeroization as enum discriminants
    /// cannot be properly zeroized - attempting to do so would be misleading
    #[zeroize(skip)]
    pub key_type: KeyType,
    /// Algorithm this key is used with
    /// Note: Enum fields are skipped from zeroization as enum discriminants
    /// cannot be properly zeroized - attempting to do so would be misleading
    #[zeroize(skip)]
    pub algorithm: EncryptionAlgorithm,
    /// Unique identifier for this key
    pub key_id: String,
    /// When this key was created
    #[zeroize(skip)]
    pub created_at: DateTime<Utc>,
    /// When this key expires (if applicable)
    #[zeroize(skip)]
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this key is active
    pub is_active: bool,
}

/// Secure byte container that zeros memory on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureBytes {
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure bytes container
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a reference to the data
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to Vec<u8> (consumes self)
    pub fn into_vec(mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
}

impl std::fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureBytes")
            .field("len", &self.data.len())
            .finish()
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for SecureBytes {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

impl Zeroize for SecureBytes {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Parameters for key derivation functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    /// Algorithm to use for key derivation
    pub algorithm: KeyDerivationAlgorithm,
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Number of iterations (for PBKDF2)
    pub iterations: Option<u32>,
    /// Memory cost (for Argon2/Scrypt)
    pub memory_cost: Option<u32>,
    /// Time cost (for Argon2)
    pub time_cost: Option<u32>,
    /// Parallelism (for Argon2)
    pub parallelism: Option<u32>,
    /// Output key length
    pub key_length: usize,
}

impl KeyDerivationParams {
    /// Create default parameters for Argon2id
    pub fn argon2id_default(salt: Vec<u8>) -> Self {
        Self {
            algorithm: KeyDerivationAlgorithm::Argon2id,
            salt,
            iterations: None,
            memory_cost: Some(65536), // 64 MB
            time_cost: Some(3),
            parallelism: Some(1),
            key_length: 32, // 256 bits
        }
    }

    /// Create default parameters for PBKDF2
    pub fn pbkdf2_default(salt: Vec<u8>) -> Self {
        Self {
            algorithm: KeyDerivationAlgorithm::Pbkdf2Sha256,
            salt,
            iterations: Some(100_000),
            memory_cost: None,
            time_cost: None,
            parallelism: None,
            key_length: 32, // 256 bits
        }
    }

    /// Create default parameters for Scrypt
    pub fn scrypt_default(salt: Vec<u8>) -> Self {
        Self {
            algorithm: KeyDerivationAlgorithm::Scrypt,
            salt,
            iterations: None,
            memory_cost: Some(8), // r parameter (block size)
            time_cost: Some(15),  // log_n parameter (2^15 = 32768)
            parallelism: Some(1), // p parameter
            key_length: 32,       // 256 bits
        }
    }
}

impl EncryptionKey {
    /// Create a new encryption key
    pub fn new(
        key_data: Vec<u8>,
        key_type: KeyType,
        algorithm: EncryptionAlgorithm,
        key_id: String,
    ) -> Self {
        Self {
            key_data: SecureBytes::new(key_data),
            key_type,
            algorithm,
            key_id,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        }
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the key is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Get the key data as bytes
    pub fn key_bytes(&self) -> &[u8] {
        self.key_data.as_slice()
    }
}

impl EncryptedData {
    /// Create new encrypted data container
    pub fn new(
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        tag: Option<Vec<u8>>,
        metadata: EncryptionMetadata,
    ) -> Self {
        Self {
            ciphertext,
            nonce,
            tag,
            metadata,
        }
    }

    /// Get the total size of the encrypted data
    pub fn total_size(&self) -> usize {
        self.ciphertext.len() + self.nonce.len() + self.tag.as_ref().map_or(0, |t| t.len())
    }
}

impl EncryptionMetadata {
    /// Create new encryption metadata
    pub fn new(algorithm: EncryptionAlgorithm, key_id: String) -> Self {
        Self {
            algorithm,
            key_id,
            encrypted_at: Utc::now(),
            version: 1,
            aad: None,
            salt: None,
        }
    }

    /// Add additional authenticated data
    pub fn with_aad(mut self, aad: Vec<u8>) -> Self {
        self.aad = Some(aad);
        self
    }

    /// Add salt information
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = Some(salt);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_bytes_zeroize() {
        let data = vec![1, 2, 3, 4, 5];
        let secure = SecureBytes::new(data.clone());
        assert_eq!(secure.as_slice(), &data);
        assert_eq!(secure.len(), 5);
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_bytes_into_vec_moves_data() {
        let data = vec![1, 2, 3, 4, 5];
        let secure = SecureBytes::new(data.clone());
        let recovered = secure.into_vec();
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_secure_bytes_manual_zeroize() {
        let data = vec![1, 2, 3, 4, 5];
        let mut secure = SecureBytes::new(data);

        // Verify data is present
        assert_eq!(secure.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(secure.len(), 5);

        // Manually zeroize
        secure.zeroize();

        // Verify data is cleared (Vec::zeroize() clears the vector completely)
        assert_eq!(secure.len(), 0);
        assert!(secure.is_empty());
        assert_eq!(secure.as_slice(), &[] as &[u8]);
    }

    #[test]
    fn test_encryption_key_validity() {
        let key = EncryptionKey::new(
            vec![0; 32],
            KeyType::Symmetric,
            EncryptionAlgorithm::Aes256Gcm,
            "test-key".to_string(),
        );

        assert!(key.is_valid());
        assert!(!key.is_expired());
        assert!(key.is_active);
    }

    #[test]
    fn test_key_derivation_params() {
        let salt = vec![0; 16];
        let params = KeyDerivationParams::argon2id_default(salt.clone());

        assert_eq!(params.algorithm, KeyDerivationAlgorithm::Argon2id);
        assert_eq!(params.salt, salt);
        assert_eq!(params.key_length, 32);
    }
}
