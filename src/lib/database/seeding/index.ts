/**
 * Database seeding utilities for development
 * This module provides functionality to populate the database with sample data
 * for development and testing purposes.
 */

import {
	executeCommand,
	executeQuery,
	executeTransaction,
	formatDateForDb,
	formatDateTimeForDb,
	generateId,
	getDatabase,
} from "../connection";
import type {
	CreateAccountInput,
	CreateBudgetPeriodInput,
	CreateCategoryInput,
	CreateGoalInput,
	CreateUserInput,
} from "../types";

export interface SeedOptions {
	/** Whether to clear existing data before seeding */
	clearExisting?: boolean;
	/** Whether to seed users */
	includeUsers?: boolean;
	/** Whether to seed accounts */
	includeAccounts?: boolean;
	/** Whether to seed categories */
	includeCategories?: boolean;
	/** Whether to seed transactions */
	includeTransactions?: boolean;
	/** Whether to seed budgets */
	includeBudgets?: boolean;
	/** Whether to seed goals */
	includeGoals?: boolean;
	/** Number of transactions to generate per account */
	transactionsPerAccount?: number;
}

export const DEFAULT_SEED_OPTIONS: SeedOptions = {
	clearExisting: false,
	includeUsers: true,
	includeAccounts: true,
	includeCategories: true,
	includeTransactions: true,
	includeBudgets: true,
	includeGoals: true,
	transactionsPerAccount: 20,
};

/**
 * Main seeding function
 * @param options Seeding configuration options
 */
export async function seedDatabase(
	options: SeedOptions = DEFAULT_SEED_OPTIONS,
): Promise<void> {
	const finalOptions = { ...DEFAULT_SEED_OPTIONS, ...options };

	console.log("🌱 Starting database seeding...");

	try {
		// Ensure database connection
		await getDatabase();

		// Clear existing data if requested
		if (finalOptions.clearExisting) {
			await clearDatabase();
		}

		// Seed in dependency order
		let userId: string | null = null;

		if (finalOptions.includeUsers) {
			userId = await seedUsers();
		}

		if (finalOptions.includeCategories && userId) {
			await seedCategories(userId);
		}

		let accountIds: string[] = [];
		if (finalOptions.includeAccounts && userId) {
			accountIds = await seedAccounts(userId);
		}

		if (finalOptions.includeTransactions && userId && accountIds.length > 0) {
			await seedTransactions(
				userId,
				accountIds,
				finalOptions.transactionsPerAccount || 20,
			);
		}

		if (finalOptions.includeBudgets && userId) {
			await seedBudgets(userId);
		}

		if (finalOptions.includeGoals && userId) {
			await seedGoals(userId);
		}

		console.log("✅ Database seeding completed successfully!");
	} catch (error) {
		console.error("❌ Database seeding failed:", error);
		throw error;
	}
}

/**
 * Clear all user data from the database
 * Note: This preserves account_types and other system data
 */
export async function clearDatabase(): Promise<void> {
	console.log("🧹 Clearing existing data...");

	const clearQueries = [
		{ query: "DELETE FROM goals" },
		{ query: "DELETE FROM budgets" },
		{ query: "DELETE FROM budget_periods" },
		{ query: "DELETE FROM transfers" },
		{ query: "DELETE FROM transactions" },
		{ query: "DELETE FROM accounts" },
		{ query: "DELETE FROM categories" },
		{ query: "DELETE FROM users" },
	];

	await executeTransaction(clearQueries);
	console.log("✅ Database cleared");
}

/**
 * Seed users table
 * @returns The created user ID
 */
async function seedUsers(): Promise<string> {
	console.log("👤 Seeding users...");

	const userId = generateId();
	const userData: CreateUserInput = {
		username: "demo_user",
		email: "demo@fiscus.app",
		// Hash of "demo123" - for demo purposes only
		// Generated with: bcrypt.hashSync("demo123", 10)
		password_hash: "$2b$10$LLGogYR2SCmGEgobPPF3ce.gtyODdqyZ6117dE3A8e2cAptAkcgh6",
	};

	const query = `
    INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6)
  `;

	const now = formatDateTimeForDb(new Date());
	await executeCommand(query, [
		userId,
		userData.username,
		userData.email,
		userData.password_hash,
		now,
		now,
	]);

	console.log(`✅ Created user: ${userData.username}`);
	return userId;
}

/**
 * Seed categories table
 * @param userId The user ID to associate categories with
 */
async function seedCategories(userId: string): Promise<void> {
	console.log("📂 Seeding categories...");

	const categories: Omit<CreateCategoryInput, "user_id">[] = [
		{
			name: "Food & Dining",
			description: "Restaurants, groceries, and food-related expenses",
			is_income: false,
			is_active: true,
			color: "#ef4444",
		},
		{
			name: "Transportation",
			description: "Gas, public transit, car maintenance, and travel",
			is_income: false,
			is_active: true,
			color: "#3b82f6",
		},
		{
			name: "Entertainment",
			description: "Movies, games, hobbies, and recreational activities",
			is_income: false,
			is_active: true,
			color: "#10b981",
		},
		{
			name: "Shopping",
			description: "Clothing, electronics, and general purchases",
			is_income: false,
			is_active: true,
			color: "#f59e0b",
		},
		{
			name: "Bills & Utilities",
			description: "Rent, electricity, water, internet, and other utilities",
			is_income: false,
			is_active: true,
			color: "#8b5cf6",
		},
		{
			name: "Healthcare",
			description: "Medical expenses, insurance, and health-related costs",
			is_income: false,
			is_active: true,
			color: "#ec4899",
		},
		{
			name: "Education",
			description: "Tuition, books, courses, and educational expenses",
			is_income: false,
			is_active: true,
			color: "#06b6d4",
		},
		{
			name: "Salary",
			description: "Primary employment income",
			is_income: true,
			is_active: true,
			color: "#22c55e",
		},
		{
			name: "Freelance",
			description: "Freelance and contract work income",
			is_income: true,
			is_active: true,
			color: "#84cc16",
		},
		{
			name: "Investments",
			description: "Dividends, capital gains, and investment returns",
			is_income: true,
			is_active: true,
			color: "#eab308",
		},
		{
			name: "Other Income",
			description: "Miscellaneous income sources",
			is_income: true,
			is_active: true,
			color: "#a855f7",
		},
	];

	const queries = categories.map((category) => {
		const id = generateId();
		const now = formatDateTimeForDb(new Date());
		return {
			query: `
        INSERT INTO categories (id, user_id, name, description, is_income, is_active, color, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      `,
			params: [
				id,
				userId,
				category.name,
				category.description,
				category.is_income,
				category.is_active,
				category.color,
				now,
				now,
			],
		};
	});

	await executeTransaction(queries);

	console.log(`✅ Created ${categories.length} categories`);
}

/**
 * Seed accounts table
 * @param userId The user ID to associate accounts with
 * @returns Array of created account IDs
 */
async function seedAccounts(userId: string): Promise<string[]> {
	console.log("🏦 Seeding accounts...");

	const accounts: Omit<CreateAccountInput, "user_id">[] = [
		{
			account_type_id: "checking",
			name: "Main Checking",
			description: "Primary checking account for daily expenses",
			initial_balance: 5420.5,
			current_balance: 5420.5,
			currency: "USD",
			is_active: true,
			institution_name: "First National Bank",
			account_number: "****1234",
		},
		{
			account_type_id: "savings",
			name: "High Yield Savings",
			description: "High-interest savings account",
			initial_balance: 15750.0,
			current_balance: 15750.0,
			currency: "USD",
			is_active: true,
			institution_name: "Online Savings Bank",
			account_number: "****5678",
		},
		{
			account_type_id: "credit_card",
			name: "Rewards Credit Card",
			description: "Cashback rewards credit card",
			initial_balance: -1250.75,
			current_balance: -1250.75,
			currency: "USD",
			is_active: true,
			institution_name: "Credit Union",
			account_number: "****9012",
		},
		{
			account_type_id: "investment",
			name: "Investment Portfolio",
			description: "Diversified investment account",
			initial_balance: 42350.25,
			current_balance: 42350.25,
			currency: "USD",
			is_active: true,
			institution_name: "Investment Firm",
			account_number: "****3456",
		},
	];

	const accountIds: string[] = [];
	const queries = accounts.map((account) => {
		const id = generateId();
		accountIds.push(id);
		const now = formatDateTimeForDb(new Date());
		return {
			query: `
        INSERT INTO accounts (
          id, user_id, account_type_id, name, description, 
          initial_balance, current_balance, currency, is_active,
          institution_name, account_number, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
      `,
			params: [
				id,
				userId,
				account.account_type_id,
				account.name,
				account.description,
				account.initial_balance,
				account.current_balance,
				account.currency,
				account.is_active,
				account.institution_name,
				account.account_number,
				now,
				now,
			],
		};
	});

	await executeTransaction(queries);

	console.log(`✅ Created ${accounts.length} accounts`);
	return accountIds;
}

/**
 * Seed transactions table
 * @param userId The user ID to associate transactions with
 * @param accountIds Array of account IDs to create transactions for
 * @param transactionsPerAccount Number of transactions to create per account
 */
async function seedTransactions(
	userId: string,
	accountIds: string[],
	transactionsPerAccount: number,
): Promise<void> {
	console.log("💳 Seeding transactions...");

	// Get categories for realistic transaction categorization
	const categoriesResult = await executeQuery<{
		id: string;
		name: string;
		is_income: boolean;
	}>("SELECT id, name, is_income FROM categories WHERE user_id = $1", [userId]);
	const categories = categoriesResult || [];
	const expenseCategories = categories.filter((c) => !c.is_income);
	const incomeCategories = categories.filter((c) => c.is_income);

	const transactions: Array<{
		query: string;
		params: unknown[];
	}> = [];

	// Sample transaction templates
	const expenseTemplates = [
		{
			description: "Grocery Store Purchase",
			merchant: "Whole Foods Market",
			categoryName: "Food & Dining",
		},
		{
			description: "Gas Station Fill-up",
			merchant: "Shell Station",
			categoryName: "Transportation",
		},
		{
			description: "Coffee Shop",
			merchant: "Starbucks",
			categoryName: "Food & Dining",
		},
		{
			description: "Movie Tickets",
			merchant: "AMC Theaters",
			categoryName: "Entertainment",
		},
		{
			description: "Online Shopping",
			merchant: "Amazon",
			categoryName: "Shopping",
		},
		{
			description: "Restaurant Dinner",
			merchant: "Local Bistro",
			categoryName: "Food & Dining",
		},
		{
			description: "Uber Ride",
			merchant: "Uber",
			categoryName: "Transportation",
		},
		{
			description: "Gym Membership",
			merchant: "Fitness Center",
			categoryName: "Healthcare",
		},
		{
			description: "Electric Bill",
			merchant: "Power Company",
			categoryName: "Bills & Utilities",
		},
		{
			description: "Internet Bill",
			merchant: "ISP Provider",
			categoryName: "Bills & Utilities",
		},
	];

	const incomeTemplates = [
		{
			description: "Salary Deposit",
			merchant: "Employer Corp",
			categoryName: "Salary",
		},
		{
			description: "Freelance Payment",
			merchant: "Client LLC",
			categoryName: "Freelance",
		},
		{
			description: "Investment Dividend",
			merchant: "Brokerage Firm",
			categoryName: "Investments",
		},
		{
			description: "Side Project Income",
			merchant: "Online Platform",
			categoryName: "Other Income",
		},
	];

	for (const accountId of accountIds) {
		// Generate transactions for the past 30 days
		const startDate = new Date();
		startDate.setDate(startDate.getDate() - 30);

		for (let i = 0; i < transactionsPerAccount; i++) {
			// Random date within the past 30 days
			const transactionDate = new Date(startDate);
			transactionDate.setDate(
				startDate.getDate() + Math.floor(Math.random() * 30),
			);

			// 80% chance of expense, 20% chance of income
			const isExpense = Math.random() < 0.8;
			const templates = isExpense ? expenseTemplates : incomeTemplates;
			const availableCategories = isExpense
				? expenseCategories
				: incomeCategories;

			if (availableCategories.length === 0) continue;

			const template = templates[Math.floor(Math.random() * templates.length)];
			const category =
				availableCategories.find((c) => c.name === template.categoryName) ||
				availableCategories[
					Math.floor(Math.random() * availableCategories.length)
				];

			// Generate realistic amounts
			let amount: number;
			if (isExpense) {
				// Expenses: $5 to $500, with most being smaller amounts
				amount = -(Math.random() < 0.7
					? Math.random() * 100 + 5
					: // 70% chance: $5-$105
						Math.random() * 400 + 100); // 30% chance: $100-$500
			} else {
				// Income: $500 to $5000
				amount = Math.random() * 4500 + 500;
			}

			// Round to 2 decimal places
			amount = Math.round(amount * 100) / 100;

			const transactionId = generateId();
			const now = formatDateTimeForDb(new Date());

			transactions.push({
				query: `
          INSERT INTO transactions (
            id, user_id, account_id, category_id, amount, description,
            transaction_date, transaction_type, status, merchant,
            created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        `,
				params: [
					transactionId,
					userId,
					accountId,
					category.id,
					amount,
					template.description,
					formatDateForDb(transactionDate),
					isExpense ? "expense" : "income",
					"completed",
					template.merchant,
					now,
					now,
				],
			});
		}
	}

	// Execute all transaction inserts in batches to avoid overwhelming the database
	const batchSize = 50;
	for (let i = 0; i < transactions.length; i += batchSize) {
		const batch = transactions.slice(i, i + batchSize);
		await executeTransaction(batch);
	}

	console.log(`✅ Created ${transactions.length} transactions`);
}

/**
 * Seed budget periods and budgets
 * @param userId The user ID to associate budgets with
 */
async function seedBudgets(userId: string): Promise<void> {
	console.log("📊 Seeding budgets...");

	// Create current month budget period
	const now = new Date();
	const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
	const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);

	const budgetPeriodId = generateId();
	const budgetPeriodData: CreateBudgetPeriodInput = {
		user_id: userId,
		name: `${startOfMonth.toLocaleString("default", { month: "long" })} ${startOfMonth.getFullYear()}`,
		start_date: formatDateForDb(startOfMonth),
		end_date: formatDateForDb(endOfMonth),
		is_active: true,
	};

	// Insert budget period
	const budgetPeriodQuery = `
    INSERT INTO budget_periods (id, user_id, name, start_date, end_date, is_active, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
  `;
	const timestamp = formatDateTimeForDb(new Date());
	await executeCommand(budgetPeriodQuery, [
		budgetPeriodId,
		budgetPeriodData.user_id,
		budgetPeriodData.name,
		budgetPeriodData.start_date,
		budgetPeriodData.end_date,
		budgetPeriodData.is_active,
		timestamp,
		timestamp,
	]);

	// Get expense categories for budget creation
	const categoriesResult = await executeQuery<{ id: string; name: string }>(
		"SELECT id, name FROM categories WHERE user_id = $1 AND is_income = 0",
		[userId],
	);
	const expenseCategories = categoriesResult || [];

	// Create budgets for major expense categories
	const budgetAllocations = [
		{ categoryName: "Food & Dining", allocated: 600 },
		{ categoryName: "Transportation", allocated: 300 },
		{ categoryName: "Entertainment", allocated: 200 },
		{ categoryName: "Shopping", allocated: 400 },
		{ categoryName: "Bills & Utilities", allocated: 800 },
		{ categoryName: "Healthcare", allocated: 150 },
	];

	const budgetQueries = budgetAllocations
		.map((allocation) => {
			const category = expenseCategories.find(
				(c) => c.name === allocation.categoryName,
			);
			if (!category) return null;

			const budgetId = generateId();
			return {
				query: `
          INSERT INTO budgets (
            id, user_id, budget_period_id, category_id,
            allocated_amount, spent_amount, created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        `,
				params: [
					budgetId,
					userId,
					budgetPeriodId,
					category.id,
					allocation.allocated,
					0, // spent_amount starts at 0
					timestamp,
					timestamp,
				],
			};
		})
		.filter(Boolean) as Array<{ query: string; params: unknown[] }>;

	if (budgetQueries.length > 0) {
		await executeTransaction(budgetQueries);
	}

	console.log(`✅ Created budget period and ${budgetQueries.length} budgets`);
}

/**
 * Seed goals table
 * @param userId The user ID to associate goals with
 */
async function seedGoals(userId: string): Promise<void> {
	console.log("🎯 Seeding goals...");

	const goals: Omit<CreateGoalInput, "user_id">[] = [
		{
			name: "Emergency Fund",
			description: "Build an emergency fund covering 6 months of expenses",
			target_amount: 15000,
			current_amount: 8500,
			target_date: formatDateForDb(
				new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
			), // 1 year from now
			priority: 5,
			status: "active",
			category: "emergency_fund",
		},
		{
			name: "Vacation to Europe",
			description: "Save for a 2-week European vacation",
			target_amount: 5000,
			current_amount: 1200,
			target_date: formatDateForDb(
				new Date(Date.now() + 180 * 24 * 60 * 60 * 1000),
			), // 6 months from now
			priority: 3,
			status: "active",
			category: "vacation",
		},
		{
			name: "New Car Down Payment",
			description: "Save for a down payment on a new car",
			target_amount: 8000,
			current_amount: 3500,
			target_date: formatDateForDb(
				new Date(Date.now() + 270 * 24 * 60 * 60 * 1000),
			), // 9 months from now
			priority: 4,
			status: "active",
			category: "car",
		},
		{
			name: "Home Improvement",
			description: "Kitchen renovation project",
			target_amount: 12000,
			current_amount: 2800,
			target_date: formatDateForDb(
				new Date(Date.now() + 450 * 24 * 60 * 60 * 1000),
			), // 15 months from now
			priority: 2,
			status: "active",
			category: "house",
		},
	];

	const queries = goals.map((goal) => {
		const id = generateId();
		const now = formatDateTimeForDb(new Date());
		return {
			query: `
        INSERT INTO goals (
          id, user_id, name, description, target_amount, current_amount,
          target_date, priority, status, category, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
      `,
			params: [
				id,
				userId,
				goal.name,
				goal.description,
				goal.target_amount,
				goal.current_amount,
				goal.target_date,
				goal.priority,
				goal.status,
				goal.category,
				now,
				now,
			],
		};
	});

	await executeTransaction(queries);

	console.log(`✅ Created ${goals.length} goals`);
}
