/**
 * Tests for database hooks
 * These tests verify the correct return types and behavior of the hooks
 */

import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAccountOperations, useTransactionOperations } from "../hooks";

// Mock the database service
vi.mock("../index", async (importOriginal) => {
	const actual = await importOriginal<typeof import("../index")>();
	return {
		...actual,
		databaseService: {
			accounts: {
				create: vi.fn(),
				update: vi.fn(),
				delete: vi.fn(),
				updateBalance: vi.fn(),
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
			expect(typeof result.current.updateBalance).toBe("function");
			expect(typeof result.current.loading).toBe("boolean");
			expect(result.current.error).toBeNull();
		});

		it("should resolve deleteAccount with correct shape on success", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return true (successful deletion)
			vi.mocked(databaseService.accounts.delete).mockResolvedValue(true);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount resolves with the expected shape
			const testId = "test-account-id";
			let deleteResult: { id: string; deleted: boolean } | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteAccount(testId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toEqual({
				id: testId,
				deleted: true,
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.accounts.delete).toHaveBeenCalledWith(testId);
		});

		it("should resolve deleteAccount with correct shape on failure", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return false (failed deletion)
			vi.mocked(databaseService.accounts.delete).mockResolvedValue(false);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount resolves with the expected shape even on failure
			const testId = "test-account-id";
			let deleteResult: { id: string; deleted: boolean } | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteAccount(testId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toEqual({
				id: testId,
				deleted: false,
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.accounts.delete).toHaveBeenCalledWith(testId);
		});

		it("should handle deleteAccount errors properly", async () => {
			const { databaseService, DatabaseError } = await import("../index");

			// Mock the delete method to throw a DatabaseError
			const errorMessage = "Failed to delete account";
			const databaseError = new DatabaseError(
				errorMessage,
				"Database connection lost",
				"DB_CONNECTION_ERROR",
			);
			vi.mocked(databaseService.accounts.delete).mockRejectedValue(
				databaseError,
			);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount properly throws the error
			const testId = "test-account-id";

			await act(async () => {
				await expect(result.current.deleteAccount(testId)).rejects.toThrow(
					DatabaseError,
				);
				await expect(result.current.deleteAccount(testId)).rejects.toThrow(
					errorMessage,
				);
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.accounts.delete).toHaveBeenCalledWith(testId);
		});

		it("should handle deleteAccount generic errors properly", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to throw a generic error
			const errorMessage = "Network timeout";
			const genericError = new Error(errorMessage);
			vi.mocked(databaseService.accounts.delete).mockRejectedValue(
				genericError,
			);

			const { result } = renderHook(() => useAccountOperations());

			// Test that deleteAccount properly throws the error
			const testId = "test-account-id";

			await act(async () => {
				await expect(result.current.deleteAccount(testId)).rejects.toThrow(
					Error,
				);
				await expect(result.current.deleteAccount(testId)).rejects.toThrow(
					errorMessage,
				);
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.accounts.delete).toHaveBeenCalledWith(testId);
		});
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

		it("should resolve deleteTransaction with correct shape on success", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return true (successful deletion)
			vi.mocked(databaseService.transactions.delete).mockResolvedValue(true);

			const { result } = renderHook(() => useTransactionOperations());

			// Test that deleteTransaction resolves with the expected shape
			const testId = "test-transaction-id";
			let deleteResult: { id: string; deleted: boolean } | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteTransaction(testId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toEqual({
				id: testId,
				deleted: true,
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.transactions.delete).toHaveBeenCalledWith(testId);
		});

		it("should resolve deleteTransaction with correct shape on failure", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return false (failed deletion)
			vi.mocked(databaseService.transactions.delete).mockResolvedValue(false);

			const { result } = renderHook(() => useTransactionOperations());

			// Test that deleteTransaction resolves with the expected shape even on failure
			const testId = "test-transaction-id";
			let deleteResult: { id: string; deleted: boolean } | undefined;

			await act(async () => {
				deleteResult = await result.current.deleteTransaction(testId);
			});

			expect(deleteResult).toBeDefined();
			expect(deleteResult).toEqual({
				id: testId,
				deleted: false,
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.transactions.delete).toHaveBeenCalledWith(testId);
		});

		it("should handle deleteTransaction errors properly", async () => {
			const { databaseService, DatabaseError } = await import("../index");

			// Mock the delete method to throw a DatabaseError
			const errorMessage = "Failed to delete transaction";
			const databaseError = new DatabaseError(
				errorMessage,
				"Database connection lost",
				"DB_CONNECTION_ERROR",
			);
			vi.mocked(databaseService.transactions.delete).mockRejectedValue(
				databaseError,
			);

			const { result } = renderHook(() => useTransactionOperations());

			// Test that deleteTransaction properly throws the error
			const testId = "test-transaction-id";

			await act(async () => {
				await expect(result.current.deleteTransaction(testId)).rejects.toThrow(
					DatabaseError,
				);
				await expect(result.current.deleteTransaction(testId)).rejects.toThrow(
					errorMessage,
				);
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.transactions.delete).toHaveBeenCalledWith(testId);
		});

		it("should handle deleteTransaction generic errors properly", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to throw a generic error
			const errorMessage = "Network timeout";
			const genericError = new Error(errorMessage);
			vi.mocked(databaseService.transactions.delete).mockRejectedValue(
				genericError,
			);

			const { result } = renderHook(() => useTransactionOperations());

			// Test that deleteTransaction properly throws the error
			const testId = "test-transaction-id";

			await act(async () => {
				await expect(result.current.deleteTransaction(testId)).rejects.toThrow(
					Error,
				);
				await expect(result.current.deleteTransaction(testId)).rejects.toThrow(
					errorMessage,
				);
			});

			// Verify the database service was called with correct parameters
			expect(databaseService.transactions.delete).toHaveBeenCalledWith(testId);
		});
	});
});
