/**
 * Tests for useEncryption hooks
 *
 * These tests verify the error handling improvements and general functionality
 * of the encryption hooks, particularly the safe error type conversion.
 */

import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EncryptionError } from "../../api/encryption";
import {
	EncryptionAlgorithm,
	encryptionApi,
	KeyType,
} from "../../api/encryption";
import {
	useDecryptData,
	useEncryptData,
	useEncryptionStats,
	useEncryptionUtils,
	useGenerateKey,
	useRotateKeys,
} from "../useEncryption";

// Mock the encryption API
vi.mock("../../api/encryption", () => ({
	encryptionApi: {
		encryptFinancialData: vi.fn(),
		decryptFinancialData: vi.fn(),
		generateEncryptionKey: vi.fn(),
		rotateUserKeys: vi.fn(),
		getEncryptionStats: vi.fn(),
	},
	encodeToBase64: vi.fn((data: string) => btoa(data)),
	decodeFromBase64: vi.fn((data: string) => atob(data)),
	formatStats: vi.fn(),
	generateSalt: vi.fn(),
	isValidAlgorithm: vi.fn(),
	isValidKeyType: vi.fn(),
	EncryptionApiClient: vi.fn(),
	// Export the enums for use in tests
	EncryptionAlgorithm: {
		Aes256Gcm: "aes_256_gcm",
		ChaCha20Poly1305: "chacha20_poly1305",
		Rsa4096: "rsa_4096",
		Ed25519: "ed25519",
		X25519: "x25519",
	},
	KeyType: {
		Symmetric: "symmetric",
		PublicKey: "public_key",
		PrivateKey: "private_key",
		DerivationKey: "derivation_key",
		MasterKey: "master_key",
	},
}));

describe("useEncryption hooks", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe("useEncryptData", () => {
		it("should handle successful encryption", async () => {
			const mockResponse = {
				encrypted_data: "encrypted-data",
				nonce: "nonce",
				algorithm: EncryptionAlgorithm.Aes256Gcm,
				key_id: "key-123",
				encrypted_at: "2023-01-01T00:00:00Z",
			};

			vi.mocked(encryptionApi.encryptFinancialData).mockResolvedValue(
				mockResponse,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				const response = await result.current.encryptData(
					"test-data",
					"user-123",
					"transaction",
				);
				expect(response).toEqual(mockResponse);
			});

			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
		});

		it("should handle EncryptionError properly", async () => {
			const encryptionError: EncryptionError = {
				type: "encryption",
				message: "Encryption failed",
				context: "Test context",
			};

			vi.mocked(encryptionApi.encryptFinancialData).mockRejectedValue(
				encryptionError,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				try {
					await result.current.encryptData(
						"test-data",
						"user-123",
						"transaction",
					);
				} catch (error) {
					expect(error).toEqual(encryptionError);
				}
			});

			expect(result.current.loading).toBe(false);
			expect(result.current.error).toEqual(encryptionError);
		});

		it("should convert Error instances to EncryptionError", async () => {
			const standardError = new Error("Network error");
			vi.mocked(encryptionApi.encryptFinancialData).mockRejectedValue(
				standardError,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				try {
					await result.current.encryptData(
						"test-data",
						"user-123",
						"transaction",
					);
				} catch (error) {
					expect(error).toEqual({
						type: "encryption",
						message: "Network error",
						context: undefined, // Error name is "Error", so context is undefined
					});
				}
			});

			expect(result.current.error).toEqual({
				type: "encryption",
				message: "Network error",
				context: undefined,
			});
		});

		it("should convert custom Error types with context", async () => {
			const customError = new TypeError("Invalid input type");
			vi.mocked(encryptionApi.encryptFinancialData).mockRejectedValue(
				customError,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				try {
					await result.current.encryptData(
						"test-data",
						"user-123",
						"transaction",
					);
				} catch (error) {
					expect(error).toEqual({
						type: "encryption",
						message: "Invalid input type",
						context: "TypeError", // Custom error name is preserved
					});
				}
			});

			expect(result.current.error).toEqual({
				type: "encryption",
				message: "Invalid input type",
				context: "TypeError",
			});
		});

		it("should handle string errors", async () => {
			const stringError = "Something went wrong";
			vi.mocked(encryptionApi.encryptFinancialData).mockRejectedValue(
				stringError,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				try {
					await result.current.encryptData(
						"test-data",
						"user-123",
						"transaction",
					);
				} catch (error) {
					expect(error).toEqual({
						type: "encryption",
						message: "Something went wrong",
					});
				}
			});
		});

		it("should handle unknown error types", async () => {
			const unknownError = { someProperty: "value" };
			vi.mocked(encryptionApi.encryptFinancialData).mockRejectedValue(
				unknownError,
			);

			const { result } = renderHook(() => useEncryptData());

			await act(async () => {
				try {
					await result.current.encryptData(
						"test-data",
						"user-123",
						"transaction",
					);
				} catch (error) {
					expect(error).toEqual({
						type: "encryption",
						message: "Unknown encryption error",
						context: "object",
					});
				}
			});
		});
	});

	describe("useDecryptData", () => {
		it("should handle successful decryption", async () => {
			const mockResponse = {
				data: btoa("decrypted-data"), // Base64 encoded
				decrypted_at: "2023-01-01T00:00:00Z",
			};

			vi.mocked(encryptionApi.decryptFinancialData).mockResolvedValue(
				mockResponse,
			);

			const { result } = renderHook(() => useDecryptData());

			await act(async () => {
				const response = await result.current.decryptData(
					"encrypted-data",
					"nonce",
					EncryptionAlgorithm.Aes256Gcm,
					"key-123",
					"user-123",
					"transaction",
				);
				expect(response.data).toBe("decrypted-data"); // Should be decoded
			});

			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
		});

		it("should convert errors safely", async () => {
			const error = new TypeError("Invalid input");
			vi.mocked(encryptionApi.decryptFinancialData).mockRejectedValue(error);

			const { result } = renderHook(() => useDecryptData());

			await act(async () => {
				try {
					await result.current.decryptData(
						"encrypted-data",
						"nonce",
						EncryptionAlgorithm.Aes256Gcm,
						"key-123",
						"user-123",
						"transaction",
					);
				} catch (thrownError) {
					expect(thrownError).toEqual({
						type: "encryption",
						message: "Invalid input",
						context: "TypeError",
					});
				}
			});
		});
	});

	describe("useGenerateKey", () => {
		it("should handle successful key generation", async () => {
			const mockResponse = {
				key_id: "key-123",
				algorithm: EncryptionAlgorithm.Aes256Gcm,
				key_type: KeyType.Symmetric,
				created_at: "2023-01-01T00:00:00Z",
			};

			vi.mocked(encryptionApi.generateEncryptionKey).mockResolvedValue(
				mockResponse,
			);

			const { result } = renderHook(() => useGenerateKey());

			await act(async () => {
				const response = await result.current.generateKey(
					"user-123",
					EncryptionAlgorithm.Aes256Gcm,
				);
				expect(response).toEqual(mockResponse);
			});

			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
		});

		it("should convert errors safely", async () => {
			const error = {
				message: "Key generation failed",
				type: "key_management",
			};
			vi.mocked(encryptionApi.generateEncryptionKey).mockRejectedValue(error);

			const { result } = renderHook(() => useGenerateKey());

			await act(async () => {
				try {
					await result.current.generateKey(
						"user-123",
						EncryptionAlgorithm.Aes256Gcm,
					);
				} catch (thrownError) {
					expect(thrownError).toEqual(error);
				}
			});
		});
	});

	describe("useRotateKeys", () => {
		it("should handle successful key rotation", async () => {
			vi.mocked(encryptionApi.rotateUserKeys).mockResolvedValue(true);

			const { result } = renderHook(() => useRotateKeys());

			await act(async () => {
				const success = await result.current.rotateKeys("user-123");
				expect(success).toBe(true);
			});

			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
		});

		it("should convert errors safely", async () => {
			const error = null; // Test null error
			vi.mocked(encryptionApi.rotateUserKeys).mockRejectedValue(error);

			const { result } = renderHook(() => useRotateKeys());

			await act(async () => {
				try {
					await result.current.rotateKeys("user-123");
				} catch (thrownError) {
					expect(thrownError).toEqual({
						type: "encryption",
						message: "Unknown encryption error",
						context: "object",
					});
				}
			});
		});
	});

	describe("useEncryptionStats", () => {
		it("should handle successful stats fetch", async () => {
			const mockStats = {
				total_keys: 10,
				active_keys: 8,
				rotated_keys: 2,
				encryption_operations: 50,
				decryption_operations: 45,
				key_derivation_operations: 5,
				last_key_rotation: "2023-01-01T00:00:00Z",
			};

			vi.mocked(encryptionApi.getEncryptionStats).mockResolvedValue(mockStats);

			const { result } = renderHook(() => useEncryptionStats());

			// Wait for the auto-fetch to complete
			await act(async () => {
				await new Promise((resolve) => setTimeout(resolve, 0));
			});

			expect(result.current.stats).toEqual(mockStats);
			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
		});

		it("should convert errors safely", async () => {
			const error = undefined; // Test undefined error
			vi.mocked(encryptionApi.getEncryptionStats).mockRejectedValue(error);

			const { result } = renderHook(() => useEncryptionStats());

			// Wait for the auto-fetch to complete
			await act(async () => {
				await new Promise((resolve) => setTimeout(resolve, 0));
			});

			expect(result.current.error).toEqual({
				type: "encryption",
				message: "Unknown encryption error",
				context: "undefined",
			});
		});
	});

	describe("useEncryptionUtils", () => {
		it("should provide utility functions", () => {
			const { result } = renderHook(() => useEncryptionUtils());

			expect(typeof result.current.validateData).toBe("function");
			expect(typeof result.current.generateSecureId).toBe("function");
			expect(typeof result.current.formatEncryptionError).toBe("function");
		});

		it("should validate data correctly", () => {
			const { result } = renderHook(() => useEncryptionUtils());

			expect(result.current.validateData("valid data")).toBe(true);
			expect(result.current.validateData("")).toBe(false);

			// Test large data (over 1MB)
			const largeData = "x".repeat(1024 * 1024 + 1);
			expect(result.current.validateData(largeData)).toBe(false);
		});

		it("should format encryption errors correctly", () => {
			const { result } = renderHook(() => useEncryptionUtils());

			const error: EncryptionError = {
				type: "validation",
				message: "Invalid input",
			};

			const formatted = result.current.formatEncryptionError(error);
			expect(formatted).toBe("Validation error: Invalid input");
		});
	});
});
