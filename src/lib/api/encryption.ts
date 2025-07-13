/**
 * TypeScript interfaces for the Fiscus encryption service
 *
 * These interfaces mirror the Rust function signatures and provide
 * type-safe access to encryption operations from the frontend.
 */

import { invoke } from "@tauri-apps/api/tauri";

// Encryption algorithm types
export enum EncryptionAlgorithm {
	Aes256Gcm = "aes_256_gcm",
	ChaCha20Poly1305 = "chacha20_poly1305",
	Rsa4096 = "rsa_4096",
	Ed25519 = "ed25519",
	X25519 = "x25519",
}

// Key types
export enum KeyType {
	Symmetric = "symmetric",
	PublicKey = "public_key",
	PrivateKey = "private_key",
	DerivationKey = "derivation_key",
	MasterKey = "master_key",
}

// Key derivation algorithms
export enum KeyDerivationAlgorithm {
	Argon2id = "argon2id",
	Pbkdf2Sha256 = "pbkdf2_sha256",
	Scrypt = "scrypt",
	HkdfSha256 = "hkdf_sha256",
}

// Request interfaces
export interface EncryptDataRequest {
	user_id: string;
	data_type: string;
	data: string; // Base64 encoded data
}

export interface DecryptDataRequest {
	user_id: string;
	data_type: string;
	encrypted_data: string; // Base64 encoded
	nonce: string; // Base64 encoded
	algorithm: EncryptionAlgorithm;
	key_id: string;
}

export interface GenerateKeyRequest {
	user_id: string;
	algorithm: EncryptionAlgorithm;
}

export interface RotateKeysRequest {
	user_id: string;
}

export interface DeriveKeyRequest {
	password: string;
	algorithm: KeyDerivationAlgorithm;
	salt?: string; // Base64 encoded salt
}

export interface SignDataRequest {
	user_id: string;
	data: string; // Base64 encoded data to sign
	private_key_id: string;
	algorithm: EncryptionAlgorithm;
}

export interface VerifySignatureRequest {
	data: string; // Base64 encoded original data
	signature: string; // Base64 encoded signature
	public_key: string; // Base64 encoded public key
	algorithm: EncryptionAlgorithm;
}

// Response interfaces
export interface EncryptDataResponse {
	encrypted_data: string; // Base64 encoded
	nonce: string; // Base64 encoded
	algorithm: EncryptionAlgorithm;
	key_id: string;
	encrypted_at: string; // ISO 8601 datetime
}

export interface DecryptDataResponse {
	data: string; // Base64 encoded decrypted data
	decrypted_at: string; // ISO 8601 datetime
}

export interface GenerateKeyResponse {
	key_id: string;
	algorithm: EncryptionAlgorithm;
	key_type: KeyType;
	created_at: string; // ISO 8601 datetime
}

export interface EncryptionStatsResponse {
	total_keys: number;
	active_keys: number;
	rotated_keys: number;
	encryption_operations: number;
	decryption_operations: number;
	key_derivation_operations: number;
	last_key_rotation?: string; // ISO 8601 datetime
}

export interface DeriveKeyResponse {
	key_id: string;
	algorithm: KeyDerivationAlgorithm;
	derived_at: string; // ISO 8601 datetime
}

export interface SignDataResponse {
	signature: string; // Base64 encoded signature
	algorithm: EncryptionAlgorithm;
	signed_at: string; // ISO 8601 datetime
}

export interface VerifySignatureResponse {
	is_valid: boolean;
	algorithm: EncryptionAlgorithm;
	verified_at: string; // ISO 8601 datetime
}

// Error types
export interface EncryptionError {
	type:
		| "encryption"
		| "key_derivation"
		| "key_management"
		| "cryptographic"
		| "validation"
		| "authentication";
	message: string;
	context?: string;
}

/**
 * Encryption API client for interacting with the Tauri backend
 */
export class EncryptionApiClient {
	/**
	 * Encrypt sensitive financial data
	 */
	async encryptFinancialData(
		request: EncryptDataRequest,
	): Promise<EncryptDataResponse> {
		try {
			return await invoke<EncryptDataResponse>("encrypt_financial_data", {
				request,
			});
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Decrypt sensitive financial data
	 */
	async decryptFinancialData(
		request: DecryptDataRequest,
	): Promise<DecryptDataResponse> {
		try {
			return await invoke<DecryptDataResponse>("decrypt_financial_data", {
				request,
			});
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Generate a new encryption key
	 */
	async generateEncryptionKey(
		request: GenerateKeyRequest,
	): Promise<GenerateKeyResponse> {
		try {
			return await invoke<GenerateKeyResponse>("generate_encryption_key", {
				request,
			});
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Rotate encryption keys for a user
	 */
	async rotateUserKeys(request: RotateKeysRequest): Promise<boolean> {
		try {
			return await invoke<boolean>("rotate_user_keys", { request });
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Get encryption service statistics
	 */
	async getEncryptionStats(): Promise<EncryptionStatsResponse> {
		try {
			return await invoke<EncryptionStatsResponse>("get_encryption_stats");
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Derive a key from password
	 */
	async deriveKeyFromPassword(
		request: DeriveKeyRequest,
	): Promise<DeriveKeyResponse> {
		try {
			return await invoke<DeriveKeyResponse>("derive_key_from_password", {
				request,
			});
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Sign data with a private key
	 */
	async signData(request: SignDataRequest): Promise<SignDataResponse> {
		try {
			return await invoke<SignDataResponse>("sign_data", { request });
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Verify a signature with a public key
	 */
	async verifySignature(
		request: VerifySignatureRequest,
	): Promise<VerifySignatureResponse> {
		try {
			return await invoke<VerifySignatureResponse>("verify_signature", {
				request,
			});
		} catch (error) {
			throw this.handleError(error);
		}
	}

	/**
	 * Handle and transform errors from the backend
	 */
	private handleError(error: unknown): EncryptionError {
		// Check if it's already a properly structured EncryptionError
		if (
			typeof error === "object" &&
			error !== null &&
			"type" in error &&
			"message" in error &&
			typeof error.type === "string" &&
			typeof error.message === "string"
		) {
			// Additional validation to ensure the type is a valid EncryptionError type
			const validTypes = [
				"encryption",
				"key_derivation",
				"key_management",
				"cryptographic",
				"validation",
				"authentication",
			];

			if (validTypes.includes(error.type)) {
				return error as EncryptionError;
			}
		}

		// Convert Error instances to EncryptionError format
		if (error instanceof Error) {
			// Determine error type based on error name/type for better categorization
			let errorType: EncryptionError["type"] = "encryption";

			if (error.name.toLowerCase().includes("validation")) {
				errorType = "validation";
			} else if (error.name.toLowerCase().includes("auth")) {
				errorType = "authentication";
			} else if (error.name.toLowerCase().includes("key")) {
				errorType = "key_management";
			} else if (error.name.toLowerCase().includes("crypto")) {
				errorType = "cryptographic";
			}

			return {
				type: errorType,
				message: error.message,
				context: error.name !== "Error" ? error.name : undefined,
			};
		}

		// Handle string errors
		if (typeof error === "string") {
			return {
				type: "encryption",
				message: error,
			};
		}

		// Fallback for unknown error types
		return {
			type: "encryption",
			message: "Unknown encryption error",
			context: typeof error,
		};
	}
}

/**
 * Encode string data to base64 for transmission
 */
export function encodeToBase64(data: string): string {
	return btoa(unescape(encodeURIComponent(data)));
}

/**
 * Decode base64 data to string
 */
export function decodeFromBase64(base64Data: string): string {
	return decodeURIComponent(escape(atob(base64Data)));
}

/**
 * Generate a random salt for key derivation
 */
export function generateSalt(length: number = 32): string {
	const array = new Uint8Array(length);
	crypto.getRandomValues(array);
	return btoa(String.fromCharCode(...array));
}

/**
 * Validate encryption algorithm
 */
export function isValidAlgorithm(
	algorithm: string,
): algorithm is EncryptionAlgorithm {
	return Object.values(EncryptionAlgorithm).includes(
		algorithm as EncryptionAlgorithm,
	);
}

/**
 * Validate key type
 */
export function isValidKeyType(keyType: string): keyType is KeyType {
	return Object.values(KeyType).includes(keyType as KeyType);
}

/**
 * Format encryption statistics for display
 */
export function formatStats(
	stats: EncryptionStatsResponse,
): Record<string, string> {
	return {
		"Total Keys": stats.total_keys.toLocaleString(),
		"Active Keys": stats.active_keys.toLocaleString(),
		"Rotated Keys": stats.rotated_keys.toLocaleString(),
		"Encryption Operations": stats.encryption_operations.toLocaleString(),
		"Decryption Operations": stats.decryption_operations.toLocaleString(),
		"Key Derivation Operations":
			stats.key_derivation_operations.toLocaleString(),
		"Last Key Rotation": stats.last_key_rotation
			? new Date(stats.last_key_rotation).toLocaleString()
			: "Never",
	};
}

// Create a singleton instance for easy use
export const encryptionApi = new EncryptionApiClient();

// Types are already exported above with their interface declarations
