/**
 * Database seeding tests
 * Tests the seeding functionality to ensure it works correctly
 */

import { afterAll, beforeAll, beforeEach, describe, expect, it } from "vitest";
import { clearDatabase, executeQuery, getDatabase, seedDatabase } from "../..";

describe("Database Seeding", () => {
	beforeAll(async () => {
		// Ensure database connection
		await getDatabase();
	});

	beforeEach(async () => {
		// Clear database before each test
		await clearDatabase();
	});

	afterAll(async () => {
		// Clean up after tests
		await clearDatabase();
	});

	describe("clearDatabase", () => {
		it("should clear all user data from database", async () => {
			// First seed some data
			await seedDatabase({
				includeUsers: true,
				includeAccounts: true,
				includeCategories: true,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			// Verify data exists
			const usersBeforeClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM users",
			);
			const accountsBeforeClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM accounts",
			);
			const categoriesBeforeClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM categories",
			);

			expect(usersBeforeClear[0].count).toBeGreaterThan(0);
			expect(accountsBeforeClear[0].count).toBeGreaterThan(0);
			expect(categoriesBeforeClear[0].count).toBeGreaterThan(0);

			// Clear database
			await clearDatabase();

			// Verify data is cleared
			const usersAfterClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM users",
			);
			const accountsAfterClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM accounts",
			);
			const categoriesAfterClear = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM categories",
			);

			expect(usersAfterClear[0].count).toBe(0);
			expect(accountsAfterClear[0].count).toBe(0);
			expect(categoriesAfterClear[0].count).toBe(0);
		});

		it("should preserve account_types table", async () => {
			// Account types should exist from migration
			const accountTypesBefore = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM account_types",
			);
			expect(accountTypesBefore[0].count).toBeGreaterThan(0);

			// Clear database
			await clearDatabase();

			// Account types should still exist
			const accountTypesAfter = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM account_types",
			);
			expect(accountTypesAfter[0].count).toBe(accountTypesBefore[0].count);
		});
	});

	describe("seedDatabase", () => {
		it("should seed users when includeUsers is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: false,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const users = await executeQuery<{ username: string; email: string }>(
				"SELECT * FROM users",
			);
			expect(users).toHaveLength(1);
			expect(users[0].username).toBe("demo_user");
			expect(users[0].email).toBe("demo@fiscus.app");
		});

		it("should seed categories when includeCategories is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: false,
				includeCategories: true,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const categories = await executeQuery("SELECT * FROM categories");
			expect(categories.length).toBeGreaterThan(0);

			// Check for expected categories
			const categoryNames = (categories as Array<{ name: string }>).map(
				(c) => c.name,
			);
			expect(categoryNames).toContain("Food & Dining");
			expect(categoryNames).toContain("Transportation");
			expect(categoryNames).toContain("Salary");
		});

		it("should seed accounts when includeAccounts is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: true,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const accounts = await executeQuery("SELECT * FROM accounts");
			expect(accounts).toHaveLength(4);

			// Check for expected accounts
			const accountNames = (accounts as Array<{ name: string }>).map(
				(a) => a.name,
			);
			expect(accountNames).toContain("Main Checking");
			expect(accountNames).toContain("High Yield Savings");
			expect(accountNames).toContain("Rewards Credit Card");
			expect(accountNames).toContain("Investment Portfolio");
		});

		it("should seed transactions when includeTransactions is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: true,
				includeCategories: true,
				includeTransactions: true,
				includeBudgets: false,
				includeGoals: false,
				transactionsPerAccount: 5,
			});

			const transactions = await executeQuery<{
				user_id: string;
				account_id: string;
				category_id: string;
				amount: number;
				description: string;
				transaction_type: string;
				status: string;
			}>("SELECT * FROM transactions");
			expect(transactions.length).toBe(20); // 4 accounts × 5 transactions

			// Check transaction properties
			const transaction = transactions[0];
			expect(transaction.user_id).toBeDefined();
			expect(transaction.account_id).toBeDefined();
			expect(transaction.category_id).toBeDefined();
			expect(transaction.amount).toBeDefined();
			expect(transaction.description).toBeDefined();
			expect(transaction.transaction_type).toMatch(/^(income|expense)$/);
			expect(transaction.status).toBe("completed");
		});

		it("should seed budgets when includeBudgets is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: false,
				includeCategories: true,
				includeTransactions: false,
				includeBudgets: true,
				includeGoals: false,
			});

			const budgetPeriods = await executeQuery<{ is_active: number }>(
				"SELECT * FROM budget_periods",
			);
			expect(budgetPeriods).toHaveLength(1);
			expect(budgetPeriods[0].is_active).toBe(1);

			const budgets = await executeQuery<{
				user_id: string;
				budget_period_id: string;
				category_id: string;
				allocated_amount: number;
				spent_amount: number;
			}>("SELECT * FROM budgets");
			expect(budgets.length).toBeGreaterThan(0);

			// Check budget properties
			const budget = budgets[0];
			expect(budget.user_id).toBeDefined();
			expect(budget.budget_period_id).toBeDefined();
			expect(budget.category_id).toBeDefined();
			expect(budget.allocated_amount).toBeGreaterThan(0);
			expect(budget.spent_amount).toBe(0);
		});

		it("should seed goals when includeGoals is true", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: false,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: true,
			});

			const goals = await executeQuery<{
				name: string;
				user_id: string;
				target_amount: number;
				current_amount: number;
				priority: number;
				status: string;
			}>("SELECT * FROM goals");
			expect(goals).toHaveLength(4);

			// Check for expected goals
			const goalNames = goals.map((g) => g.name);
			expect(goalNames).toContain("Emergency Fund");
			expect(goalNames).toContain("Vacation to Europe");
			expect(goalNames).toContain("New Car Down Payment");
			expect(goalNames).toContain("Home Improvement");

			// Check goal properties
			const goal = goals[0];
			expect(goal.user_id).toBeDefined();
			expect(goal.target_amount).toBeGreaterThan(0);
			expect(goal.current_amount).toBeGreaterThanOrEqual(0);
			expect(goal.priority).toBeGreaterThanOrEqual(1);
			expect(goal.priority).toBeLessThanOrEqual(5);
			expect(goal.status).toBe("active");
		});

		it("should respect clearExisting option", async () => {
			// First seed some data
			await seedDatabase({
				includeUsers: true,
				includeAccounts: false,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const usersAfterFirstSeed = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM users",
			);
			expect(usersAfterFirstSeed[0].count).toBe(1);

			// Seed again with clearExisting: false (should not clear)
			await seedDatabase({
				clearExisting: false,
				includeUsers: true,
				includeAccounts: false,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const usersAfterSecondSeed = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM users",
			);
			expect(usersAfterSecondSeed[0].count).toBe(2); // Should have 2 users now

			// Seed again with clearExisting: true (should clear first)
			await seedDatabase({
				clearExisting: true,
				includeUsers: true,
				includeAccounts: false,
				includeCategories: false,
				includeTransactions: false,
				includeBudgets: false,
				includeGoals: false,
			});

			const usersAfterThirdSeed = await executeQuery<{ count: number }>(
				"SELECT COUNT(*) as count FROM users",
			);
			expect(usersAfterThirdSeed[0].count).toBe(1); // Should be back to 1 user
		});
	});

	describe("data relationships", () => {
		it("should maintain proper foreign key relationships", async () => {
			await seedDatabase({
				includeUsers: true,
				includeAccounts: true,
				includeCategories: true,
				includeTransactions: true,
				includeBudgets: true,
				includeGoals: true,
				transactionsPerAccount: 2,
			});

			// Check that all transactions have valid user_id, account_id, and category_id
			const invalidTransactions = await executeQuery(`
				SELECT t.id 
				FROM transactions t
				LEFT JOIN users u ON t.user_id = u.id
				LEFT JOIN accounts a ON t.account_id = a.id
				LEFT JOIN categories c ON t.category_id = c.id
				WHERE u.id IS NULL OR a.id IS NULL OR c.id IS NULL
			`);
			expect(invalidTransactions).toHaveLength(0);

			// Check that all budgets have valid relationships
			const invalidBudgets = await executeQuery(`
				SELECT b.id 
				FROM budgets b
				LEFT JOIN users u ON b.user_id = u.id
				LEFT JOIN budget_periods bp ON b.budget_period_id = bp.id
				LEFT JOIN categories c ON b.category_id = c.id
				WHERE u.id IS NULL OR bp.id IS NULL OR c.id IS NULL
			`);
			expect(invalidBudgets).toHaveLength(0);
		});
	});
});
