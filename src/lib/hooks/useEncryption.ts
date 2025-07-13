/**
 * React hooks for encryption operations
 *
 * This module provides React hooks that integrate with the encryption service,
 * offering a convenient way to perform encryption operations from React components.
 */

import { useCallback, useEffect, useState } from "react";
import {
	type DecryptDataRequest,
	decodeFromBase64,
	type EncryptDataRequest,
	type EncryptionAlgorithm,
	EncryptionApiClient,
	type EncryptionError,
	type EncryptionStatsResponse,
	encodeToBase64,
	encryptionApi,
	formatStats,
	type GenerateKeyRequest,
	generateSalt,
	isValidAlgorithm,
	isValidKeyType,
	type RotateKeysRequest,
} from "../api/encryption";
import { secureStorage } from "../services/secureStorage";

// Hook state types
interface EncryptionState<T> {
	data: T | null;
	loading: boolean;
	error: EncryptionError | null;
}

interface EncryptionOperationState {
	loading: boolean;
	error: EncryptionError | null;
}

/**
 * Safely converts unknown errors to EncryptionError format
 * This prevents unsafe casting and ensures consistent error handling
 */
function toEncryptionError(error: unknown): EncryptionError {
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

/**
 * Hook for encrypting financial data
 */
export function useEncryptData() {
	const [state, setState] = useState<EncryptionOperationState>({
		loading: false,
		error: null,
	});

	const encryptData = useCallback(
		async (data: string, userId: string, dataType: string) => {
			setState({ loading: true, error: null });

			try {
				const base64Data = encodeToBase64(data);
				const request: EncryptDataRequest = {
					user_id: userId,
					data_type: dataType,
					data: base64Data,
				};

				const response = await encryptionApi.encryptFinancialData(request);
				setState({ loading: false, error: null });
				return response;
			} catch (error) {
				const encryptionError = toEncryptionError(error);
				setState({ loading: false, error: encryptionError });
				throw encryptionError;
			}
		},
		[],
	);

	return {
		encryptData,
		loading: state.loading,
		error: state.error,
	};
}

/**
 * Hook for decrypting financial data
 */
export function useDecryptData() {
	const [state, setState] = useState<EncryptionOperationState>({
		loading: false,
		error: null,
	});

	const decryptData = useCallback(
		async (
			encryptedData: string,
			nonce: string,
			algorithm: EncryptionAlgorithm,
			keyId: string,
			userId: string,
			dataType: string,
		) => {
			setState({ loading: true, error: null });

			try {
				const request: DecryptDataRequest = {
					user_id: userId,
					data_type: dataType,
					encrypted_data: encryptedData,
					nonce,
					algorithm,
					key_id: keyId,
				};

				const response = await encryptionApi.decryptFinancialData(request);
				const decryptedData = decodeFromBase64(response.data);

				setState({ loading: false, error: null });
				return {
					...response,
					data: decryptedData, // Return decoded string instead of base64
				};
			} catch (error) {
				const encryptionError = toEncryptionError(error);
				setState({ loading: false, error: encryptionError });
				throw encryptionError;
			}
		},
		[],
	);

	return {
		decryptData,
		loading: state.loading,
		error: state.error,
	};
}

/**
 * Hook for generating encryption keys
 */
export function useGenerateKey() {
	const [state, setState] = useState<EncryptionOperationState>({
		loading: false,
		error: null,
	});

	const generateKey = useCallback(
		async (userId: string, algorithm: EncryptionAlgorithm) => {
			setState({ loading: true, error: null });

			try {
				const request: GenerateKeyRequest = {
					user_id: userId,
					algorithm,
				};

				const response = await encryptionApi.generateEncryptionKey(request);
				setState({ loading: false, error: null });
				return response;
			} catch (error) {
				const encryptionError = toEncryptionError(error);
				setState({ loading: false, error: encryptionError });
				throw encryptionError;
			}
		},
		[],
	);

	return {
		generateKey,
		loading: state.loading,
		error: state.error,
	};
}

/**
 * Hook for rotating user keys
 */
export function useRotateKeys() {
	const [state, setState] = useState<EncryptionOperationState>({
		loading: false,
		error: null,
	});

	const rotateKeys = useCallback(async (userId: string) => {
		setState({ loading: true, error: null });

		try {
			const request: RotateKeysRequest = {
				user_id: userId,
			};

			const success = await encryptionApi.rotateUserKeys(request);
			setState({ loading: false, error: null });
			return success;
		} catch (error) {
			const encryptionError = toEncryptionError(error);
			setState({ loading: false, error: encryptionError });
			throw encryptionError;
		}
	}, []);

	return {
		rotateKeys,
		loading: state.loading,
		error: state.error,
	};
}

/**
 * Hook for fetching encryption statistics
 */
export function useEncryptionStats() {
	const [state, setState] = useState<EncryptionState<EncryptionStatsResponse>>({
		data: null,
		loading: false,
		error: null,
	});

	const fetchStats = useCallback(async () => {
		setState((prev) => ({ ...prev, loading: true, error: null }));

		try {
			const stats = await encryptionApi.getEncryptionStats();
			setState({ data: stats, loading: false, error: null });
			return stats;
		} catch (error) {
			const encryptionError = toEncryptionError(error);
			setState((prev) => ({ ...prev, loading: false, error: encryptionError }));
			throw encryptionError;
		}
	}, []);

	// Auto-fetch on mount
	useEffect(() => {
		let mounted = true;

		const doFetch = async () => {
			if (mounted) {
				await fetchStats();
			}
		};

		doFetch();

		return () => {
			mounted = false;
		};
	}, [fetchStats]);

	return {
		stats: state.data,
		loading: state.loading,
		error: state.error,
		refetch: fetchStats,
	};
}

/**
 * Hook for secure data handling with automatic encryption/decryption
 */
export function useSecureData<T>(
	userId: string,
	dataType: string,
	initialData?: T,
) {
	const [data, setData] = useState<T | null>(initialData || null);
	const [isEncrypted, setIsEncrypted] = useState(false);
	const {
		encryptData,
		loading: encrypting,
		error: encryptError,
	} = useEncryptData();
	const {
		decryptData,
		loading: decrypting,
		error: decryptError,
	} = useDecryptData();

	const secureStore = useCallback(
		async (value: T) => {
			try {
				const serializedData = JSON.stringify(value);
				const encryptedResponse = await encryptData(
					serializedData,
					userId,
					dataType,
				);

				// Store encrypted data securely using Tauri backend instead of localStorage
				await secureStorage.store({
					user_id: userId,
					data_type: dataType,
					encrypted_data: encryptedResponse.encrypted_data,
					nonce: encryptedResponse.nonce,
					algorithm: encryptedResponse.algorithm,
					key_id: encryptedResponse.key_id,
				});

				setData(value);
				setIsEncrypted(true);
				return encryptedResponse;
			} catch (error) {
				console.error("Failed to securely store data:", error);
				throw error;
			}
		},
		[encryptData, userId, dataType],
	);

	const secureRetrieve = useCallback(async (): Promise<T | null> => {
		try {
			// Retrieve encrypted data from secure backend storage
			const storedData = await secureStorage.retrieve({
				user_id: userId,
				data_type: dataType,
			});

			if (!storedData) {
				return null;
			}

			const decryptedResponse = await decryptData(
				storedData.encrypted_data,
				storedData.nonce,
				storedData.algorithm,
				storedData.key_id,
				userId,
				dataType,
			);

			const parsedData = JSON.parse(decryptedResponse.data) as T;
			setData(parsedData);
			setIsEncrypted(false);
			return parsedData;
		} catch (error) {
			console.error("Failed to securely retrieve data:", error);
			throw error;
		}
	}, [decryptData, userId, dataType]);

	const secureClear = useCallback(async () => {
		try {
			// Delete encrypted data from secure backend storage
			await secureStorage.delete({
				user_id: userId,
				data_type: dataType,
			});
			setData(null);
			setIsEncrypted(false);
		} catch (error) {
			console.error("Failed to securely clear data:", error);
			// Still clear local state even if backend deletion fails
			setData(null);
			setIsEncrypted(false);
		}
	}, [userId, dataType]);

	return {
		data,
		isEncrypted,
		secureStore,
		secureRetrieve,
		secureClear,
		loading: encrypting || decrypting,
		error: encryptError || decryptError,
	};
}

/**
 * Hook for encryption utilities and helpers
 */
export function useEncryptionUtils() {
	const validateData = useCallback((data: string): boolean => {
		const byteSize = new TextEncoder().encode(data).length;
		return data.length > 0 && byteSize <= 1024 * 1024; // 1MB limit
	}, []);

	const generateSecureId = useCallback((): string => {
		return crypto.randomUUID();
	}, []);

	const formatEncryptionError = useCallback(
		(error: EncryptionError): string => {
			switch (error.type) {
				case "encryption":
					return `Encryption failed: ${error.message}`;
				case "key_derivation":
					return `Key derivation failed: ${error.message}`;
				case "key_management":
					return `Key management error: ${error.message}`;
				case "cryptographic":
					return `Cryptographic operation failed: ${error.message}`;
				case "validation":
					return `Validation error: ${error.message}`;
				case "authentication":
					return `Authentication error: ${error.message}`;
				default:
					return `Unknown error: ${error.message}`;
			}
		},
		[],
	);

	return {
		validateData,
		generateSecureId,
		formatEncryptionError,
		encodeToBase64,
		decodeFromBase64,
		generateSalt,
		isValidAlgorithm,
		isValidKeyType,
		formatStats,
	};
}

// Export all hooks
export { encryptionApi, EncryptionApiClient };
