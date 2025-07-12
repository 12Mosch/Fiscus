/// Utility functions and helpers for the encryption service
///
/// This module provides common utilities used throughout the encryption system,
/// including secure random number generation, memory protection, and data conversion helpers.
use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use tracing::{debug, error, instrument};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::types::EncryptionResult;
use crate::error::FiscusError;

/// Secure random number generator for cryptographic operations
#[derive(Debug)]
pub struct SecureRandom {
    rng: OsRng,
}

impl SecureRandom {
    /// Create a new secure random number generator
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing secure random number generator");
        Ok(Self { rng: OsRng })
    }

    /// Generate cryptographically secure random bytes
    #[instrument(skip(self), fields(length = length))]
    pub fn generate_bytes(&mut self, length: usize) -> EncryptionResult<Vec<u8>> {
        if length == 0 {
            return Err(FiscusError::InvalidInput(
                "Cannot generate zero-length random bytes".to_string(),
            ));
        }

        if length > 1024 * 1024 {
            return Err(FiscusError::InvalidInput(
                "Requested random bytes length too large (max 1MB)".to_string(),
            ));
        }

        let mut bytes = vec![0u8; length];
        self.rng.fill_bytes(&mut bytes);

        debug!(length = length, "Generated secure random bytes");
        Ok(bytes)
    }

    /// Generate a random salt for key derivation
    pub fn generate_salt(&mut self) -> EncryptionResult<Vec<u8>> {
        self.generate_bytes(32) // 256-bit salt
    }

    /// Generate a random nonce/IV
    pub fn generate_nonce(
        &mut self,
        algorithm: super::types::EncryptionAlgorithm,
    ) -> EncryptionResult<Vec<u8>> {
        let length = match algorithm {
            super::types::EncryptionAlgorithm::Aes256Gcm => 12, // 96-bit nonce for GCM
            super::types::EncryptionAlgorithm::ChaCha20Poly1305 => 12, // 96-bit nonce
            _ => {
                return Err(FiscusError::InvalidInput(
                    "Unsupported algorithm for nonce generation".to_string(),
                ))
            }
        };

        self.generate_bytes(length)
    }

    /// Generate a random key ID
    pub fn generate_key_id(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl Default for SecureRandom {
    fn default() -> Self {
        Self::new().expect("Failed to create secure random generator")
    }
}

/// Memory protection utilities for handling sensitive data
pub struct MemoryProtection;

impl MemoryProtection {
    /// Securely clear memory containing sensitive data
    pub fn secure_clear(data: &mut [u8]) {
        data.zeroize();
    }

    /// Create a secure buffer that will be cleared on drop
    pub fn secure_buffer(size: usize) -> SecureBuffer {
        SecureBuffer::new(size)
    }

    /// Check if a memory region contains only zeros
    pub fn is_cleared(data: &[u8]) -> bool {
        data.iter().all(|&b| b == 0)
    }
}

/// A buffer that automatically clears its contents when dropped
#[derive(ZeroizeOnDrop)]
pub struct SecureBuffer {
    #[zeroize(skip)]
    data: Vec<u8>,
}

impl SecureBuffer {
    /// Create a new secure buffer with the specified size
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    /// Create a secure buffer from existing data
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a mutable reference to the buffer data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get a reference to the buffer data
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert the buffer to a Vec<u8> (consumes the buffer)
    pub fn into_vec(self) -> Vec<u8> {
        self.data.clone()
    }

    /// Resize the buffer (new bytes are zeroed)
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }

    /// Clear the buffer contents
    pub fn clear(&mut self) {
        self.data.zeroize();
    }
}

impl std::fmt::Debug for SecureBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureBuffer")
            .field("len", &self.data.len())
            .finish()
    }
}

/// Encoding utilities for converting between different data formats
pub struct EncodingUtils;

impl EncodingUtils {
    /// Encode bytes to base64 string
    pub fn encode_base64(data: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(data)
    }

    /// Decode base64 string to bytes
    pub fn decode_base64(encoded: &str) -> EncryptionResult<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| {
                error!("Base64 decoding failed: {}", e);
                FiscusError::InvalidInput(format!("Invalid base64 encoding: {e}"))
            })
    }

    /// Encode bytes to hexadecimal string
    pub fn encode_hex(data: &[u8]) -> String {
        hex::encode(data)
    }

    /// Decode hexadecimal string to bytes
    pub fn decode_hex(encoded: &str) -> EncryptionResult<Vec<u8>> {
        hex::decode(encoded).map_err(|e| {
            error!("Hex decoding failed: {}", e);
            FiscusError::InvalidInput(format!("Invalid hex encoding: {e}"))
        })
    }

    /// Convert bytes to a safe string representation for logging
    pub fn safe_display(data: &[u8], max_length: usize) -> String {
        if data.len() <= max_length {
            format!("{}...", hex::encode(&data[..std::cmp::min(8, data.len())]))
        } else {
            format!("{}... ({} bytes)", hex::encode(&data[..8]), data.len())
        }
    }
}

/// Timing-safe comparison utilities
pub struct TimingSafeComparison;

impl TimingSafeComparison {
    /// Compare two byte slices in constant time
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }

    /// Verify that two strings are equal in constant time
    pub fn verify_strings(a: &str, b: &str) -> bool {
        Self::constant_time_eq(a.as_bytes(), b.as_bytes())
    }
}

/// Validation utilities for encryption parameters
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate key length for a specific algorithm
    pub fn validate_key_length(
        key_length: usize,
        algorithm: super::types::EncryptionAlgorithm,
    ) -> EncryptionResult<()> {
        let expected_length = match algorithm {
            super::types::EncryptionAlgorithm::Aes256Gcm => 32,
            super::types::EncryptionAlgorithm::ChaCha20Poly1305 => 32,
            super::types::EncryptionAlgorithm::Rsa4096 => return Ok(()), // Variable length
            super::types::EncryptionAlgorithm::Ed25519 => 32,
            super::types::EncryptionAlgorithm::X25519 => 32,
        };

        if algorithm != super::types::EncryptionAlgorithm::Rsa4096 && key_length != expected_length
        {
            return Err(FiscusError::InvalidInput(format!(
                "Invalid key length for {algorithm:?}: expected {expected_length}, got {key_length}"
            )));
        }

        Ok(())
    }

    /// Validate nonce/IV length for a specific algorithm
    pub fn validate_nonce_length(
        nonce_length: usize,
        algorithm: super::types::EncryptionAlgorithm,
    ) -> EncryptionResult<()> {
        let expected_length = match algorithm {
            super::types::EncryptionAlgorithm::Aes256Gcm => 12,
            super::types::EncryptionAlgorithm::ChaCha20Poly1305 => 12,
            _ => return Ok(()), // Not applicable for asymmetric algorithms
        };

        if nonce_length != expected_length {
            return Err(FiscusError::InvalidInput(format!(
                "Invalid nonce length for {algorithm:?}: expected {expected_length}, got {nonce_length}"
            )));
        }

        Ok(())
    }

    /// Validate that data is not empty
    pub fn validate_non_empty(data: &[u8], field_name: &str) -> EncryptionResult<()> {
        if data.is_empty() {
            return Err(FiscusError::InvalidInput(format!(
                "{field_name} cannot be empty"
            )));
        }
        Ok(())
    }

    /// Validate data size limits
    pub fn validate_data_size(
        data: &[u8],
        max_size: usize,
        field_name: &str,
    ) -> EncryptionResult<()> {
        if data.len() > max_size {
            return Err(FiscusError::InvalidInput(format!(
                "{} size exceeds maximum limit: {} > {}",
                field_name,
                data.len(),
                max_size
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_random_generation() {
        let mut rng = SecureRandom::new().unwrap();

        let bytes1 = rng.generate_bytes(32).unwrap();
        let bytes2 = rng.generate_bytes(32).unwrap();

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_secure_buffer() {
        let mut buffer = SecureBuffer::new(16);
        assert_eq!(buffer.len(), 16);

        // Fill with test data
        buffer.as_mut_slice().copy_from_slice(&[1u8; 16]);
        assert_eq!(buffer.as_slice(), &[1u8; 16]);

        // Clear should zero the buffer
        buffer.clear();
        assert!(MemoryProtection::is_cleared(buffer.as_slice()));
    }

    #[test]
    fn test_timing_safe_comparison() {
        let a = b"secret_key_123";
        let b = b"secret_key_123";
        let c = b"secret_key_456";

        assert!(TimingSafeComparison::constant_time_eq(a, b));
        assert!(!TimingSafeComparison::constant_time_eq(a, c));
    }

    #[test]
    fn test_encoding_utils() {
        let data = b"test data";

        // Base64 roundtrip
        let encoded = EncodingUtils::encode_base64(data);
        let decoded = EncodingUtils::decode_base64(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());

        // Hex roundtrip
        let hex_encoded = EncodingUtils::encode_hex(data);
        let hex_decoded = EncodingUtils::decode_hex(&hex_encoded).unwrap();
        assert_eq!(data, hex_decoded.as_slice());
    }

    #[test]
    fn test_validation_utils() {
        use super::super::types::EncryptionAlgorithm;

        // Valid key length
        assert!(ValidationUtils::validate_key_length(32, EncryptionAlgorithm::Aes256Gcm).is_ok());

        // Invalid key length
        assert!(ValidationUtils::validate_key_length(16, EncryptionAlgorithm::Aes256Gcm).is_err());

        // Non-empty validation
        assert!(ValidationUtils::validate_non_empty(b"data", "test").is_ok());
        assert!(ValidationUtils::validate_non_empty(b"", "test").is_err());
    }
}
