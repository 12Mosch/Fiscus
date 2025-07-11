/**
 * Unit tests for AccountRepository methods
 * These tests focus on method logic without requiring database connection
 */

import { beforeEach, describe, expect, it, vi } from "vitest";
import * as connection from "../../connection";
import type { QueryOptions } from "../../types";
import { AccountRepository } from "../accounts";

// Mock the database connection module
vi.mock("../../connection", () => ({
	executeQuery: vi.fn(),
	executeCommand: vi.fn(),
}));

describe("AccountRepository", () => {
	let repository: AccountRepository;
	let mockExecuteQuery: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		repository = new AccountRepository();
		mockExecuteQuery = vi.mocked(connection.executeQuery);
		mockExecuteQuery.mockClear();
	});

	describe("findByType", () => {
		it("should construct correct query with user ID and account type ID", async () => {
			const userId = "user-123";
			const accountTypeId = "checking";
			const mockAccounts = [
				{
					id: "acc-1",
					user_id: userId,
					account_type_id: accountTypeId,
					name: "Test Account",
					current_balance: 1000,
				},
			];

			mockExecuteQuery.mockResolvedValue(mockAccounts);

			await repository.findByType(userId, accountTypeId);

			// Verify the query was called with correct parameters
			expect(mockExecuteQuery).toHaveBeenCalledWith(
				expect.stringContaining("WHERE user_id = $1 AND account_type_id = $2"),
				[userId, accountTypeId, 50], // Default limit is 50
			);
		});

		it("should respect custom query options", async () => {
			const userId = "user-123";
			const accountTypeId = "savings";
			const options: QueryOptions = {
				limit: 10,
				sort: { field: "name", direction: "asc" },
			};

			mockExecuteQuery.mockResolvedValue([]);

			await repository.findByType(userId, accountTypeId, options);

			// Verify the query includes custom limit and sorting (with quoted field name for security)
			expect(mockExecuteQuery).toHaveBeenCalledWith(
				expect.stringContaining('ORDER BY "name" ASC'),
				[userId, accountTypeId, 10],
			);
		});

		it("should use default sorting when no sort option provided", async () => {
			const userId = "user-123";
			const accountTypeId = "investment";

			mockExecuteQuery.mockResolvedValue([]);

			await repository.findByType(userId, accountTypeId);

			// Verify default sorting is applied
			expect(mockExecuteQuery).toHaveBeenCalledWith(
				expect.stringContaining("ORDER BY created_at DESC"),
				[userId, accountTypeId, 50],
			);
		});

		it("should return the accounts from executeQuery", async () => {
			const userId = "user-123";
			const accountTypeId = "credit";
			const mockAccounts = [
				{
					id: "acc-1",
					user_id: userId,
					account_type_id: accountTypeId,
					name: "Credit Card",
					current_balance: -500,
				},
				{
					id: "acc-2",
					user_id: userId,
					account_type_id: accountTypeId,
					name: "Another Credit Card",
					current_balance: -200,
				},
			];

			mockExecuteQuery.mockResolvedValue(mockAccounts);

			const result = await repository.findByType(userId, accountTypeId);

			expect(result).toEqual(mockAccounts);
		});
	});
});
