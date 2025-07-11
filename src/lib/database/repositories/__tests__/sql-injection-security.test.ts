/**
 * Security tests to verify SQL injection vulnerabilities are fixed
 * These tests ensure that malicious sort parameters cannot execute arbitrary SQL
 */

import { beforeEach, describe, expect, it, vi } from "vitest";
import * as connection from "../../connection";
import { AccountRepository } from "../accounts";
import { TransactionRepository } from "../transactions";

// Mock the database connection module
vi.mock("../../connection", () => ({
	executeQuery: vi.fn(),
	executeCommand: vi.fn(),
}));

describe("SQL Injection Security Tests", () => {
	let accountRepository: AccountRepository;
	let transactionRepository: TransactionRepository;
	let mockExecuteQuery: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		accountRepository = new AccountRepository();
		transactionRepository = new TransactionRepository();
		mockExecuteQuery = vi.mocked(connection.executeQuery);
		mockExecuteQuery.mockClear();
		mockExecuteQuery.mockResolvedValue([]);
	});

	describe("AccountRepository SQL Injection Prevention", () => {
		it("should prevent SQL injection in sort field", async () => {
			const maliciousSort = {
				field: "name; DROP TABLE accounts; --",
				direction: "asc" as "asc",
			};

			await accountRepository.findByType("user-123", "checking", {
				sort: maliciousSort,
			});

			// Verify that the malicious field was not used and default was applied
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY "created_at"');
			expect(query).not.toContain("DROP TABLE");
			expect(query).not.toContain("--");
		});

		it("should prevent SQL injection in sort direction", async () => {
			const maliciousSort = {
				field: "name",
				direction: "asc; DROP TABLE accounts; --" as "asc",
			};

			await accountRepository.findByType("user-123", "checking", {
				sort: maliciousSort,
			});

			// Verify that the malicious direction was sanitized to DESC (default fallback)
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY "name" DESC');
			expect(query).not.toContain("DROP TABLE");
			expect(query).not.toContain("--");
		});

		it("should only allow whitelisted sort fields", async () => {
			const invalidSort = {
				field: "malicious_field",
				direction: "asc" as "asc",
			};

			await accountRepository.findByType("user-123", "checking", {
				sort: invalidSort,
			});

			// Should fall back to default sort when field is not whitelisted
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY "created_at"');
			expect(query).not.toContain("malicious_field");
		});

		it("should properly quote valid field names", async () => {
			const validSort = {
				field: "name",
				direction: "desc" as "desc",
			};

			await accountRepository.findByType("user-123", "checking", {
				sort: validSort,
			});

			// Should properly quote the field name
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY "name" DESC');
		});
	});

	describe("TransactionRepository SQL Injection Prevention", () => {
		it("should prevent SQL injection in transaction sort field", async () => {
			const maliciousSort = {
				field: "amount; DELETE FROM transactions; --",
				direction: "desc" as "desc",
			};

			await transactionRepository.findWithDetails(
				"user-123",
				{},
				{
					sort: maliciousSort,
				},
			);

			// Verify that the malicious field was not used and falls back to default
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY t."created_at"');
			expect(query).not.toContain("DELETE FROM");
			expect(query).not.toContain("--");
		});

		it("should validate transaction sort fields against whitelist", async () => {
			const validSort = {
				field: "transaction_date",
				direction: "asc" as "asc",
			};

			await transactionRepository.findWithDetails(
				"user-123",
				{},
				{
					sort: validSort,
				},
			);

			// Should use the valid field with proper quoting
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY t."transaction_date" ASC');
		});

		it("should reject invalid transaction sort fields", async () => {
			const invalidSort = {
				field: "invalid_column",
				direction: "asc" as "asc",
			};

			await transactionRepository.findWithDetails(
				"user-123",
				{},
				{
					sort: invalidSort,
				},
			);

			// Should fall back to default sort
			const [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain('ORDER BY t."created_at"');
			expect(query).not.toContain("invalid_column");
		});
	});

	describe("Base Repository Security", () => {
		it("should log security warnings for invalid sort fields", async () => {
			const consoleSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

			const maliciousSort = {
				field: "malicious_field",
				direction: "asc" as "asc",
			};

			await accountRepository.findAll({}, { sort: maliciousSort });

			// Should log a warning about the invalid field
			expect(consoleSpy).toHaveBeenCalledWith(
				expect.stringContaining(
					"Invalid sort field attempted: malicious_field",
				),
			);

			consoleSpy.mockRestore();
		});

		it("should handle empty or null sort parameters safely", async () => {
			// Test with undefined sort
			await accountRepository.findAll({}, {});
			let [query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain("ORDER BY created_at DESC");

			mockExecuteQuery.mockClear();

			// Test with null sort
			await accountRepository.findAll({}, { sort: undefined });
			[query] = mockExecuteQuery.mock.calls[0];
			expect(query).toContain("ORDER BY created_at DESC");
		});
	});
});
