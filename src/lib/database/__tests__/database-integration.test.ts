/**
 * Database integration tests
 * Tests the complete database setup and basic operations
 */

import { afterAll, beforeAll, beforeEach, describe, expect, it } from "vitest";
import {
	databaseService,
	dbUtils,
	formatDateForDb,
	formatDateTimeForDb,
	generateId,
	getDatabaseVersion,
	isDatabaseConnected,
} from "../index";
import type { CreateAccountInput, CreateTransactionInput } from "../types";

describe("Database Integration", () => {
	beforeAll(async () => {
		// Initialize database service
		await databaseService.initialize();
	});

	afterAll(async () => {
		// Clean up database connections
		await databaseService.shutdown();
	});

	describe("Connection Management", () => {
		it("should establish database connection", async () => {
			const isConnected = await isDatabaseConnected();
			expect(isConnected).toBe(true);
		});

		it("should return database version", async () => {
			const version = await getDatabaseVersion();
			expect(typeof version).toBe("number");
			expect(version).toBeGreaterThanOrEqual(0);
		});

		it("should get health status", async () => {
			const health = await databaseService.getHealthStatus();
			expect(health).toHaveProperty("connected");
			expect(health).toHaveProperty("version");
			expect(health).toHaveProperty("timestamp");
			expect(health.connected).toBe(true);
		});
	});

	describe("Utility Functions", () => {
		it("should generate valid UUIDs", () => {
			const id1 = generateId();
			const id2 = generateId();

			expect(typeof id1).toBe("string");
			expect(typeof id2).toBe("string");
			expect(id1).not.toBe(id2);
			expect(dbUtils.isValidId(id1)).toBe(true);
			expect(dbUtils.isValidId(id2)).toBe(true);
		});

		it("should format dates correctly", () => {
			const date = new Date("2024-01-15T10:30:00Z");
			const formattedDate = formatDateForDb(date);
			const formattedDateTime = formatDateTimeForDb(date);

			expect(formattedDate).toBe("2024-01-15");
			expect(formattedDateTime).toBe("2024-01-15T10:30:00.000Z");
		});

		it("should format currency correctly", () => {
			const formatted = dbUtils.formatCurrency(1234.56);
			expect(formatted).toBe("$1,234.56");
		});

		it("should parse and stringify tags", () => {
			const tags = ["food", "grocery", "essential"];
			const stringified = dbUtils.stringifyTags(tags);
			const parsed = dbUtils.parseTags(stringified);

			expect(parsed).toEqual(tags);
		});
	});

	describe("Account Operations", () => {
		let testUserId: string;

		beforeEach(() => {
			testUserId = generateId();
		});

		it("should create an account", async () => {
			const accountData: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Test Checking Account",
				description: "Test account for integration testing",
				initial_balance: 1000.0,
				current_balance: 1000.0,
				currency: "USD",
				is_active: true,
				institution_name: "Test Bank",
			};

			const account = await databaseService.accounts.create(accountData);

			expect(account).toBeDefined();
			expect(account.id).toBeDefined();
			expect(account.name).toBe(accountData.name);
			expect(account.current_balance).toBe(accountData.current_balance);
			expect(account.user_id).toBe(testUserId);
		});

		it("should find account by id", async () => {
			// First create an account
			const accountData: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "savings",
				name: "Test Savings Account",
				initial_balance: 500.0,
				current_balance: 500.0,
				currency: "USD",
				is_active: true,
			};

			const createdAccount = await databaseService.accounts.create(accountData);

			// Then find it
			const foundAccount = await databaseService.accounts.findById(
				createdAccount.id,
			);

			expect(foundAccount).toBeDefined();
			expect(foundAccount?.id).toBe(createdAccount.id);
			expect(foundAccount?.name).toBe(accountData.name);
		});

		it("should update account balance", async () => {
			// Create account
			const accountData: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Balance Test Account",
				initial_balance: 100.0,
				current_balance: 100.0,
				currency: "USD",
				is_active: true,
			};

			const account = await databaseService.accounts.create(accountData);

			// Update balance
			const newBalance = 250.0;
			const updatedAccount = await databaseService.accounts.updateBalance(
				account.id,
				newBalance,
			);

			expect(updatedAccount.current_balance).toBe(newBalance);
		});

		it("should get account balances for user", async () => {
			// Create multiple accounts
			const account1Data: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Account 1",
				initial_balance: 100.0,
				current_balance: 100.0,
				currency: "USD",
				is_active: true,
			};

			const account2Data: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "savings",
				name: "Account 2",
				initial_balance: 200.0,
				current_balance: 200.0,
				currency: "USD",
				is_active: true,
			};

			await Promise.all([
				databaseService.accounts.create(account1Data),
				databaseService.accounts.create(account2Data),
			]);

			const balances =
				await databaseService.accounts.getAccountBalances(testUserId);

			expect(balances).toHaveLength(2);
			expect(balances.some((b) => b.account_name === "Account 1")).toBe(true);
			expect(balances.some((b) => b.account_name === "Account 2")).toBe(true);
		});
	});

	describe("Transaction Operations", () => {
		let testUserId: string;
		let testAccountId: string;

		beforeEach(async () => {
			testUserId = generateId();

			// Create a test account first
			const accountData: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Transaction Test Account",
				initial_balance: 1000.0,
				current_balance: 1000.0,
				currency: "USD",
				is_active: true,
			};

			const account = await databaseService.accounts.create(accountData);
			testAccountId = account.id;
		});

		it("should create a transaction with balance update", async () => {
			const transactionData: CreateTransactionInput = {
				user_id: testUserId,
				account_id: testAccountId,
				amount: -50.0,
				description: "Test expense transaction",
				transaction_date: formatDateForDb(new Date()),
				transaction_type: "expense",
				status: "completed",
				payee: "Test Store",
			};

			const transaction =
				await databaseService.transactions.createWithBalanceUpdate(
					transactionData,
				);

			expect(transaction).toBeDefined();
			expect(transaction.id).toBeDefined();
			expect(transaction.amount).toBe(transactionData.amount);
			expect(transaction.description).toBe(transactionData.description);
			expect(transaction.user_id).toBe(testUserId);
			expect(transaction.account_id).toBe(testAccountId);

			// Verify account balance was updated
			const updatedAccount =
				await databaseService.accounts.findById(testAccountId);
			expect(updatedAccount?.current_balance).toBe(950.0); // 1000 - 50
		});

		it("should find transactions with details", async () => {
			// Create a transaction first
			const transactionData: CreateTransactionInput = {
				user_id: testUserId,
				account_id: testAccountId,
				amount: 100.0,
				description: "Test income transaction",
				transaction_date: formatDateForDb(new Date()),
				transaction_type: "income",
				status: "completed",
			};

			await databaseService.transactions.createWithBalanceUpdate(
				transactionData,
			);

			// Find transactions with details
			const result =
				await databaseService.transactions.findWithDetails(testUserId);

			expect(result.data).toHaveLength(1);
			expect(result.data[0].description).toBe(transactionData.description);
			expect(result.data[0].account).toBeDefined();
			expect(result.data[0].account.name).toBe("Transaction Test Account");
		});

		it("should get recent transactions", async () => {
			// Create multiple transactions
			const transactions = [
				{
					user_id: testUserId,
					account_id: testAccountId,
					amount: -25.0,
					description: "Transaction 1",
					transaction_date: formatDateForDb(new Date()),
					transaction_type: "expense" as const,
					status: "completed" as const,
				},
				{
					user_id: testUserId,
					account_id: testAccountId,
					amount: -35.0,
					description: "Transaction 2",
					transaction_date: formatDateForDb(new Date()),
					transaction_type: "expense" as const,
					status: "completed" as const,
				},
			];

			await Promise.all(
				transactions.map((tx) =>
					databaseService.transactions.createWithBalanceUpdate(tx),
				),
			);

			const recent = await databaseService.transactions.getRecent(
				testUserId,
				5,
			);

			expect(recent).toHaveLength(2);
			expect(recent[0].description).toBe("Transaction 2"); // Most recent first
			expect(recent[1].description).toBe("Transaction 1");
		});
	});

	describe("Error Handling", () => {
		it("should handle non-existent record gracefully", async () => {
			const nonExistentId = generateId();
			const account = await databaseService.accounts.findById(nonExistentId);

			expect(account).toBeNull();
		});

		it("should throw error for invalid update", async () => {
			const nonExistentId = generateId();

			await expect(
				databaseService.accounts.update(nonExistentId, {
					name: "Updated Name",
				}),
			).rejects.toThrow();
		});
	});
});
