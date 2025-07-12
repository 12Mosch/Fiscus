/**
 * React hooks for encryption operations
 *
 * This module provides React hooks that integrate with the encryption service,
 * offering a convenient way to perform encryption operations from React components.
 */

import { useCallback, useEffect, useState } from "react";
import {
	type DecryptDataRequest,
	type EncryptDataRequest,
	type EncryptionAlgorithm,
	EncryptionApiClient,
	type EncryptionError,
	type EncryptionStatsResponse,
	EncryptionUtils,
	encryptionApi,
	type GenerateKeyRequest,
	type RotateKeysRequest,
} from "../api/encryption";

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
				const base64Data = EncryptionUtils.encodeToBase64(data);
				const request: EncryptDataRequest = {
					user_id: userId,
					data_type: dataType,
					data: base64Data,
				};

				const response = await encryptionApi.encryptFinancialData(request);
				setState({ loading: false, error: null });
				return response;
			} catch (error) {
				const encryptionError = error as EncryptionError;
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
				const decryptedData = EncryptionUtils.decodeFromBase64(response.data);

				setState({ loading: false, error: null });
				return {
					...response,
					data: decryptedData, // Return decoded string instead of base64
				};
			} catch (error) {
				const encryptionError = error as EncryptionError;
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
				const encryptionError = error as EncryptionError;
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
			const encryptionError = error as EncryptionError;
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
			const encryptionError = error as EncryptionError;
			setState((prev) => ({ ...prev, loading: false, error: encryptionError }));
			throw encryptionError;
		}
	}, []);

	// Auto-fetch on mount
	useEffect(() => {
		fetchStats();
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

				// Store encrypted metadata for later retrieval
				const encryptedMetadata = {
					encrypted_data: encryptedResponse.encrypted_data,
					nonce: encryptedResponse.nonce,
					algorithm: encryptedResponse.algorithm,
					key_id: encryptedResponse.key_id,
				};

				// In a real app, you'd store this in localStorage or a secure store
				localStorage.setItem(
					`secure_${dataType}_${userId}`,
					JSON.stringify(encryptedMetadata),
				);

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
			const storedMetadata = localStorage.getItem(
				`secure_${dataType}_${userId}`,
			);
			if (!storedMetadata) {
				return null;
			}

			const metadata = JSON.parse(storedMetadata);
			const decryptedResponse = await decryptData(
				metadata.encrypted_data,
				metadata.nonce,
				metadata.algorithm,
				metadata.key_id,
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

	const secureClear = useCallback(() => {
		localStorage.removeItem(`secure_${dataType}_${userId}`);
		setData(null);
		setIsEncrypted(false);
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
		return data.length > 0 && data.length <= 1024 * 1024; // 1MB limit
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
		encodeToBase64: EncryptionUtils.encodeToBase64,
		decodeFromBase64: EncryptionUtils.decodeFromBase64,
		generateSalt: EncryptionUtils.generateSalt,
		isValidAlgorithm: EncryptionUtils.isValidAlgorithm,
		isValidKeyType: EncryptionUtils.isValidKeyType,
		formatStats: EncryptionUtils.formatStats,
	};
}

// Export all hooks
export { encryptionApi, EncryptionApiClient, EncryptionUtils };
