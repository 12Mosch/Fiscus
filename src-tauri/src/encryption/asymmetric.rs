use async_trait::async_trait;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
/// Asymmetric encryption implementations for the Fiscus encryption service
///
/// This module provides asymmetric (public-key) encryption capabilities using
/// RSA-4096 and Ed25519 algorithms for secure key exchange and digital signatures.
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use tracing::{debug, error, instrument, warn};

use super::types::{
    EncryptedData, EncryptionAlgorithm, EncryptionKey, EncryptionMetadata, EncryptionResult,
    KeyType,
};
use super::utils::SecureRandom;
use crate::error::FiscusError;

/// Trait for asymmetric encryption operations
#[async_trait]
pub trait AsymmetricEncryption {
    /// Generate a new key pair
    async fn generate_keypair(&self) -> EncryptionResult<(EncryptionKey, EncryptionKey)>;

    /// Encrypt data with a public key
    async fn encrypt_with_public_key(
        &self,
        data: &[u8],
        public_key: &[u8],
    ) -> EncryptionResult<EncryptedData>;

    /// Decrypt data with a private key
    async fn decrypt_with_private_key(
        &self,
        encrypted_data: &EncryptedData,
        private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>>;

    /// Sign data with a private key
    async fn sign_data(
        &self,
        data: &[u8],
        private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>>;

    /// Verify a signature with a public key
    async fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> EncryptionResult<bool>;

    /// Get the algorithm identifier
    fn algorithm(&self) -> EncryptionAlgorithm;
}

/// RSA-4096 asymmetric encryption implementation
///
/// RSA with 4096-bit keys provides strong security for key exchange and
/// digital signatures, though it's slower than elliptic curve alternatives.
#[derive(Debug)]
pub struct RsaEncryption {
    #[allow(dead_code)]
    secure_random: SecureRandom,
}

impl RsaEncryption {
    /// Create a new RSA encryption instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing RSA-4096 encryption");
        Ok(Self {
            secure_random: SecureRandom::new()?,
        })
    }

    /// Convert RSA public key to PEM format
    fn public_key_to_pem(public_key: &RsaPublicKey) -> EncryptionResult<Vec<u8>> {
        public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map(|pem| pem.into_bytes())
            .map_err(|e| {
                error!("Failed to encode RSA public key: {}", e);
                FiscusError::Internal("Failed to encode public key".to_string())
            })
    }

    /// Convert RSA private key to PEM format
    fn private_key_to_pem(private_key: &RsaPrivateKey) -> EncryptionResult<Vec<u8>> {
        private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map(|pem| pem.as_bytes().to_vec())
            .map_err(|e| {
                error!("Failed to encode RSA private key: {}", e);
                FiscusError::Internal("Failed to encode private key".to_string())
            })
    }

    /// Parse RSA public key from PEM format
    fn public_key_from_pem(pem_data: &[u8]) -> EncryptionResult<RsaPublicKey> {
        let pem_str = std::str::from_utf8(pem_data)
            .map_err(|_| FiscusError::InvalidInput("Invalid PEM encoding".to_string()))?;

        RsaPublicKey::from_public_key_pem(pem_str).map_err(|e| {
            error!("Failed to parse RSA public key: {}", e);
            FiscusError::InvalidInput("Invalid RSA public key format".to_string())
        })
    }

    /// Parse RSA private key from PEM format
    fn private_key_from_pem(pem_data: &[u8]) -> EncryptionResult<RsaPrivateKey> {
        let pem_str = std::str::from_utf8(pem_data)
            .map_err(|_| FiscusError::InvalidInput("Invalid PEM encoding".to_string()))?;

        RsaPrivateKey::from_pkcs8_pem(pem_str).map_err(|e| {
            error!("Failed to parse RSA private key: {}", e);
            FiscusError::InvalidInput("Invalid RSA private key format".to_string())
        })
    }
}

#[async_trait]
impl AsymmetricEncryption for RsaEncryption {
    #[instrument(skip(self))]
    async fn generate_keypair(&self) -> EncryptionResult<(EncryptionKey, EncryptionKey)> {
        debug!("Generating RSA-4096 key pair");

        let mut rng = rand::rngs::OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 4096).map_err(|e| {
            error!("Failed to generate RSA private key: {}", e);
            FiscusError::Internal("Failed to generate RSA key pair".to_string())
        })?;

        let public_key = RsaPublicKey::from(&private_key);

        // Convert keys to PEM format
        let private_pem = Self::private_key_to_pem(&private_key)?;
        let public_pem = Self::public_key_to_pem(&public_key)?;

        let key_id = uuid::Uuid::new_v4().to_string();

        let private_key_obj = EncryptionKey::new(
            private_pem,
            KeyType::PrivateKey,
            EncryptionAlgorithm::Rsa4096,
            format!("{key_id}-private"),
        );

        let public_key_obj = EncryptionKey::new(
            public_pem,
            KeyType::PublicKey,
            EncryptionAlgorithm::Rsa4096,
            format!("{key_id}-public"),
        );

        debug!(
            private_key_id = %private_key_obj.key_id,
            public_key_id = %public_key_obj.key_id,
            "RSA-4096 key pair generated successfully"
        );

        Ok((private_key_obj, public_key_obj))
    }

    #[instrument(skip(self, data, public_key), fields(data_len = data.len()))]
    async fn encrypt_with_public_key(
        &self,
        data: &[u8],
        public_key: &[u8],
    ) -> EncryptionResult<EncryptedData> {
        // RSA can only encrypt small amounts of data
        if data.len() > 446 {
            // 4096/8 - 42 (PKCS#1 v1.5 padding)
            return Err(FiscusError::InvalidInput(
                "Data too large for RSA encryption (max 446 bytes)".to_string(),
            ));
        }

        let rsa_public_key = Self::public_key_from_pem(public_key)?;
        let mut rng = rand::rngs::OsRng;

        let ciphertext = rsa_public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, data)
            .map_err(|e| {
                error!("RSA encryption failed: {}", e);
                FiscusError::Internal("RSA encryption failed".to_string())
            })?;

        let metadata =
            EncryptionMetadata::new(EncryptionAlgorithm::Rsa4096, "rsa-public".to_string());

        debug!(
            ciphertext_len = ciphertext.len(),
            "RSA encryption completed successfully"
        );

        Ok(EncryptedData::new(
            ciphertext,
            Vec::new(), // RSA doesn't use nonces
            None,
            metadata,
        ))
    }

    #[instrument(skip(self, encrypted_data, private_key), fields(ciphertext_len = encrypted_data.ciphertext.len()))]
    async fn decrypt_with_private_key(
        &self,
        encrypted_data: &EncryptedData,
        private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        if encrypted_data.metadata.algorithm != EncryptionAlgorithm::Rsa4096 {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for RSA decryption".to_string(),
            ));
        }

        let rsa_private_key = Self::private_key_from_pem(private_key.key_bytes())?;

        let plaintext = rsa_private_key
            .decrypt(Pkcs1v15Encrypt, &encrypted_data.ciphertext)
            .map_err(|e| {
                error!("RSA decryption failed: {}", e);
                FiscusError::Authentication("RSA decryption failed".to_string())
            })?;

        debug!(
            plaintext_len = plaintext.len(),
            "RSA decryption completed successfully"
        );

        Ok(plaintext)
    }

    async fn sign_data(
        &self,
        _data: &[u8],
        _private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        // TODO: For RSA signatures, we'd typically use PSS or PKCS#1 v1.5
        // This is a simplified implementation
        warn!("RSA signing not fully implemented in this version");
        Err(FiscusError::Internal(
            "RSA signing not implemented".to_string(),
        ))
    }

    async fn verify_signature(
        &self,
        _data: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> EncryptionResult<bool> {
        warn!("RSA signature verification not fully implemented in this version");
        Err(FiscusError::Internal(
            "RSA signature verification not implemented".to_string(),
        ))
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Rsa4096
    }
}

/// Ed25519 asymmetric encryption implementation
///
/// Ed25519 provides fast, secure digital signatures and key exchange
/// using elliptic curve cryptography.
#[derive(Debug)]
pub struct Ed25519Encryption {
    #[allow(dead_code)]
    secure_random: SecureRandom,
}

impl Ed25519Encryption {
    /// Create a new Ed25519 encryption instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing Ed25519 encryption");
        Ok(Self {
            secure_random: SecureRandom::new()?,
        })
    }
}

#[async_trait]
impl AsymmetricEncryption for Ed25519Encryption {
    #[instrument(skip(self))]
    async fn generate_keypair(&self) -> EncryptionResult<(EncryptionKey, EncryptionKey)> {
        debug!("Generating Ed25519 key pair");

        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let key_id = uuid::Uuid::new_v4().to_string();

        let private_key = EncryptionKey::new(
            signing_key.to_bytes().to_vec(),
            KeyType::PrivateKey,
            EncryptionAlgorithm::Ed25519,
            format!("{key_id}-private"),
        );

        let public_key = EncryptionKey::new(
            verifying_key.to_bytes().to_vec(),
            KeyType::PublicKey,
            EncryptionAlgorithm::Ed25519,
            format!("{key_id}-public"),
        );

        debug!(
            private_key_id = %private_key.key_id,
            public_key_id = %public_key.key_id,
            "Ed25519 key pair generated successfully"
        );

        Ok((private_key, public_key))
    }

    async fn encrypt_with_public_key(
        &self,
        _data: &[u8],
        _public_key: &[u8],
    ) -> EncryptionResult<EncryptedData> {
        // Ed25519 is primarily for signatures, not encryption
        // For encryption, we'd typically use X25519 key exchange + symmetric encryption
        Err(FiscusError::InvalidInput(
            "Ed25519 is for signatures, not encryption. Use X25519 for key exchange.".to_string(),
        ))
    }

    async fn decrypt_with_private_key(
        &self,
        _encrypted_data: &EncryptedData,
        _private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        Err(FiscusError::InvalidInput(
            "Ed25519 is for signatures, not encryption".to_string(),
        ))
    }

    #[instrument(skip(self, data, private_key), fields(data_len = data.len()))]
    async fn sign_data(
        &self,
        data: &[u8],
        private_key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        if private_key.algorithm != EncryptionAlgorithm::Ed25519 {
            return Err(FiscusError::InvalidInput(
                "Key algorithm mismatch for Ed25519 signing".to_string(),
            ));
        }

        if private_key.key_bytes().len() != 32 {
            return Err(FiscusError::InvalidInput(
                "Invalid Ed25519 private key length".to_string(),
            ));
        }

        let key_bytes: [u8; 32] = private_key.key_bytes().try_into().map_err(|_| {
            FiscusError::InvalidInput("Invalid Ed25519 private key length".to_string())
        })?;

        let signing_key = SigningKey::from_bytes(&key_bytes);

        let signature = signing_key.sign(data);

        debug!(
            signature_len = signature.to_bytes().len(),
            "Ed25519 signature created successfully"
        );

        Ok(signature.to_bytes().to_vec())
    }

    #[instrument(skip(self, data, signature, public_key), fields(data_len = data.len(), sig_len = signature.len()))]
    async fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> EncryptionResult<bool> {
        if public_key.len() != 32 {
            return Err(FiscusError::InvalidInput(
                "Invalid Ed25519 public key length".to_string(),
            ));
        }

        if signature.len() != 64 {
            return Err(FiscusError::InvalidInput(
                "Invalid Ed25519 signature length".to_string(),
            ));
        }

        let public_key_bytes: [u8; 32] = public_key.try_into().map_err(|_| {
            FiscusError::InvalidInput("Invalid Ed25519 public key length".to_string())
        })?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes).map_err(|e| {
            error!("Invalid Ed25519 public key: {}", e);
            FiscusError::InvalidInput("Invalid Ed25519 public key".to_string())
        })?;

        let signature_bytes: [u8; 64] = signature.try_into().map_err(|_| {
            FiscusError::InvalidInput("Invalid Ed25519 signature length".to_string())
        })?;

        let signature_obj = Signature::from_bytes(&signature_bytes);

        let is_valid = verifying_key.verify(data, &signature_obj).is_ok();

        debug!(
            is_valid = is_valid,
            "Ed25519 signature verification completed"
        );
        Ok(is_valid)
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Ed25519
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rsa_keypair_generation() {
        let rsa = RsaEncryption::new().unwrap();
        let (private_key, public_key) = rsa.generate_keypair().await.unwrap();

        assert_eq!(private_key.algorithm, EncryptionAlgorithm::Rsa4096);
        assert_eq!(public_key.algorithm, EncryptionAlgorithm::Rsa4096);
        assert_eq!(private_key.key_type, KeyType::PrivateKey);
        assert_eq!(public_key.key_type, KeyType::PublicKey);
    }

    #[tokio::test]
    async fn test_ed25519_keypair_generation() {
        let ed25519 = Ed25519Encryption::new().unwrap();
        let (private_key, public_key) = ed25519.generate_keypair().await.unwrap();

        assert_eq!(private_key.algorithm, EncryptionAlgorithm::Ed25519);
        assert_eq!(public_key.algorithm, EncryptionAlgorithm::Ed25519);
        assert_eq!(private_key.key_bytes().len(), 32);
        assert_eq!(public_key.key_bytes().len(), 32);
    }

    #[tokio::test]
    async fn test_ed25519_signature_roundtrip() {
        let ed25519 = Ed25519Encryption::new().unwrap();
        let (private_key, public_key) = ed25519.generate_keypair().await.unwrap();
        let data = b"test message for signing";

        let signature = ed25519.sign_data(data, &private_key).await.unwrap();
        let is_valid = ed25519
            .verify_signature(data, &signature, public_key.key_bytes())
            .await
            .unwrap();

        assert!(is_valid);

        // Test with wrong data
        let wrong_data = b"wrong message";
        let is_invalid = ed25519
            .verify_signature(wrong_data, &signature, public_key.key_bytes())
            .await
            .unwrap();
        assert!(!is_invalid);
    }
}
