import { invoke } from "@tauri-apps/api/tauri";
import type { EncryptionAlgorithm } from "../api/encryption";

/**
 * Secure storage service that uses Tauri backend for encrypted data persistence
 * This replaces localStorage usage to prevent XSS vulnerabilities
 */

export interface SecureStoreRequest {
	user_id: string;
	data_type: string;
	encrypted_data: string; // Base64 encoded encrypted data
	nonce: string; // Base64 encoded nonce
	algorithm: EncryptionAlgorithm; // Encryption algorithm
	key_id: string;
}

export interface SecureStoreResponse {
	stored: boolean;
	storage_key: string;
	stored_at: string;
}

export interface SecureRetrieveRequest {
	user_id: string;
	data_type: string;
}

export interface SecureRetrieveResponse {
	encrypted_data: string; // Base64 encoded encrypted data
	nonce: string; // Base64 encoded nonce
	algorithm: EncryptionAlgorithm; // Encryption algorithm
	key_id: string;
	stored_at: string;
}

export interface SecureDeleteRequest {
	user_id: string;
	data_type: string;
}

export interface SecureDeleteResponse {
	deleted: boolean;
	deleted_at: string;
}

/**
 * Secure storage service class
 */
export class SecureStorageService {
	/**
	 * Store encrypted data securely using Tauri backend
	 */
	async store(request: SecureStoreRequest): Promise<SecureStoreResponse> {
		try {
			const response = await invoke<SecureStoreResponse>("secure_store", {
				request,
			});
			return response;
		} catch (error) {
			console.error("Failed to store data securely:", error);
			throw new Error(`Secure storage failed: ${error}`);
		}
	}

	/**
	 * Retrieve encrypted data securely from Tauri backend
	 */
	async retrieve(
		request: SecureRetrieveRequest,
	): Promise<SecureRetrieveResponse | null> {
		try {
			const response = await invoke<SecureRetrieveResponse>("secure_retrieve", {
				request,
			});
			return response;
		} catch (error) {
			// If data is not found, return null instead of throwing
			if (error && typeof error === "string" && error.includes("NotFound")) {
				return null;
			}
			console.error("Failed to retrieve data securely:", error);
			throw new Error(`Secure retrieval failed: ${error}`);
		}
	}

	/**
	 * Delete encrypted data securely from Tauri backend
	 */
	async delete(request: SecureDeleteRequest): Promise<SecureDeleteResponse> {
		try {
			const response = await invoke<SecureDeleteResponse>("secure_delete", {
				request,
			});
			return response;
		} catch (error) {
			console.error("Failed to delete data securely:", error);
			throw new Error(`Secure deletion failed: ${error}`);
		}
	}

	/**
	 * Generate a storage key for the given user and data type
	 * This matches the backend implementation
	 */
	generateStorageKey(userId: string, dataType: string): string {
		return `secure_${dataType}_${userId}`;
	}
}

/**
 * Singleton instance of the secure storage service
 */
export const secureStorage = new SecureStorageService();

/**
 * Legacy localStorage-like interface for easy migration
 * This provides a familiar API while using secure backend storage
 */
export class SecureStorageLegacyAdapter {
	private storage = secureStorage;

	/**
	 * Store data with a key (async operation)
	 */
	async setItem(key: string, value: string, userId: string): Promise<void> {
		// Parse the key to extract data type
		const dataType = this.extractDataTypeFromKey(key);

		// For legacy compatibility, we assume the value is already encrypted metadata
		try {
			const metadata = JSON.parse(value);
			await this.storage.store({
				user_id: userId,
				data_type: dataType,
				encrypted_data: metadata.encrypted_data,
				nonce: metadata.nonce,
				algorithm: metadata.algorithm,
				key_id: metadata.key_id,
			});
		} catch (error) {
			throw new Error(`Failed to store item with key ${key}: ${error}`);
		}
	}

	/**
	 * Retrieve data by key (async operation)
	 */
	async getItem(key: string, userId: string): Promise<string | null> {
		const dataType = this.extractDataTypeFromKey(key);

		try {
			const response = await this.storage.retrieve({
				user_id: userId,
				data_type: dataType,
			});

			if (!response) {
				return null;
			}

			// Return in the same format as the original localStorage implementation
			return JSON.stringify({
				encrypted_data: response.encrypted_data,
				nonce: response.nonce,
				algorithm: response.algorithm,
				key_id: response.key_id,
			});
		} catch (error) {
			console.error(`Failed to retrieve item with key ${key}:`, error);
			return null;
		}
	}

	/**
	 * Remove data by key (async operation)
	 */
	async removeItem(key: string, userId: string): Promise<void> {
		const dataType = this.extractDataTypeFromKey(key);

		try {
			await this.storage.delete({
				user_id: userId,
				data_type: dataType,
			});
		} catch (error) {
			console.error(`Failed to remove item with key ${key}:`, error);
			// Don't throw for removal failures to match localStorage behavior
		}
	}

	/**
	 * Extract data type from legacy localStorage key format
	 * Expected format: "secure_{dataType}_{userId}"
	 */
	private extractDataTypeFromKey(key: string): string {
		const parts = key.split("_");
		if (parts.length >= 3 && parts[0] === "secure") {
			// Remove 'secure' prefix and userId suffix to get dataType
			return parts.slice(1, -1).join("_");
		}
		// Fallback to using the key as-is
		return key;
	}
}

/**
 * Legacy adapter instance for easy migration from localStorage
 */
export const secureStorageLegacy = new SecureStorageLegacyAdapter();
