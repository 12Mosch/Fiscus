/**
 * Tests for database hooks
 * These tests verify the correct return types and behavior of the hooks
 * Updated to test the migrated API service-based hooks
 */

import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { apiService } from "../../api-service";
import { useAccountOperations, useTransactionOperations } from "../hooks";

// Mock the API service
vi.mock("../../api-service", async (importOriginal) => {
	const actual = await importOriginal<typeof import("../../api-service")>();
	return {
		...actual,
		apiService: {
			accounts: {
				create: vi.fn(),
				update: vi.fn(),
				delete: vi.fn(),
			},
			transactions: {
				createWithBalanceUpdate: vi.fn(),
				update: vi.fn(),
				delete: vi.fn(),
			},
		},
	};
});

describe("Database Hooks", () => {
	describe("useAccountOperations", () => {
		it("should return correct function signatures", () => {
			const { result } = renderHook(() => useAccountOperations());

			expect(typeof result.current.createAccount).toBe("function");
			expect(typeof result.current.updateAccount).toBe("function");
			expect(typeof result.current.deleteAccount).toBe("function");
			expect(typeof result.current.loading).toBe("boolean");
			expect(result.current.error).toBeNull();
		});

		it("should resolve deleteAccount with correct shape on success", async () => {
			const { apiService } = await import("../../api-service");

			// Mock the delete method to return true (successful deletion)
			vi.mocked(apiService.accounts.delete).mockResolvedValue(true);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount resolves with the expected shape
			const testId = "test-account-id";
			const testUserId = "test-user-id";
			let deleteResult: boolean | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteAccount(testId, testUserId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toBe(true);

			// Verify the API service was called with correct parameters
			expect(apiService.accounts.delete).toHaveBeenCalledWith(
				testId,
				testUserId,
			);
		});

		it("should resolve deleteAccount with correct shape on failure", async () => {
			// Mock the delete method to return false (failed deletion)
			vi.mocked(apiService.accounts.delete).mockResolvedValue(false);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount resolves with the expected shape even on failure
			const testId = "test-account-id";
			const testUserId = "test-user-id";
			let deleteResult: boolean | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteAccount(testId, testUserId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toBe(false);

			// Verify the API service was called with correct parameters
			expect(apiService.accounts.delete).toHaveBeenCalledWith(
				testId,
				testUserId,
			);
		});

		// Additional tests for error handling can be added here
		// when the API service error handling is fully implemented
	});

	describe("useTransactionOperations", () => {
		it("should return correct function signatures", () => {
			const { result } = renderHook(() => useTransactionOperations());

			expect(typeof result.current.createTransaction).toBe("function");
			expect(typeof result.current.updateTransaction).toBe("function");
			expect(typeof result.current.deleteTransaction).toBe("function");
			expect(typeof result.current.loading).toBe("boolean");
			expect(result.current.error).toBeNull();
		});

		// Additional transaction operation tests can be added here
		// when the API service transaction operations are fully implemented
	});
});
