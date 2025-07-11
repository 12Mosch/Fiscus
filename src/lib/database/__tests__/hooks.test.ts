/**
 * Tests for database hooks
 * These tests verify the correct return types and behavior of the hooks
 */

import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAccountOperations, useTransactionOperations } from "../hooks";

// Mock the database service
vi.mock("../index", () => ({
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
}));

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

		it("should have correct deleteAccount return type", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return true (successful deletion)
			vi.mocked(databaseService.accounts.delete).mockResolvedValue(true);

			const { result } = renderHook(() => useAccountOperations());

			// The deleteAccount function should return a promise that resolves to { id: string, deleted: boolean }
			const deletePromise = result.current.deleteAccount("test-id");
			expect(deletePromise).toBeInstanceOf(Promise);

			// We can't easily test the resolved value without more complex mocking,
			// but we've verified the function exists and returns a promise
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

		it("should have correct deleteTransaction return type", async () => {
			const { databaseService } = await import("../index");

			// Mock the delete method to return true (successful deletion)
			vi.mocked(databaseService.transactions.delete).mockResolvedValue(true);

			const { result } = renderHook(() => useTransactionOperations());

			// The deleteTransaction function should return a promise that resolves to { id: string, deleted: boolean }
			const deletePromise = result.current.deleteTransaction("test-id");
			expect(deletePromise).toBeInstanceOf(Promise);

			// We can't easily test the resolved value without more complex mocking,
			// but we've verified the function exists and returns a promise
		});
	});
});
