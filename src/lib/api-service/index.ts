/**
 * API-based service layer for Fiscus application
 * Replaces the direct database service with secure Tauri API calls
 *
 * This service provides the same interface as the old database service
 * but uses the secure Tauri API commands instead of direct database access.
 */

import { apiClient } from "@/api/client";
import type {
	Account,
	AccountFilters,
	CreateAccountRequest,
	CreateTransactionRequest,
	Transaction,
	TransactionFilters,
	UpdateAccountRequest,
	UpdateTransactionRequest,
} from "@/types/api";

// Types for compatibility with existing database service
export interface QueryOptions {
	limit?: number;
	offset?: number;
	sort?: {
		field: string;
		direction: "asc" | "desc";
	};
}

export interface QueryResult<T> {
	data: T[];
	total: number;
	page: number;
	limit: number;
}

export interface DashboardSummary {
	total_assets: number;
	total_liabilities: number;
	net_worth: number;
	monthly_income: number;
	monthly_expenses: number;
	recent_transactions: Transaction[];
	account_balances: Array<{
		account_id: string;
		account_name: string;
		current_balance: number;
		currency: string;
	}>;
	top_categories: Array<{
		category_id: string;
		category_name: string;
		total_spent: number;
		transaction_count: number;
	}>;
}

/**
 * Account repository using secure API calls
 */
export class ApiAccountRepository {
	/**
	 * Create a new account
	 */
	async create(input: CreateAccountRequest): Promise<Account> {
		return await apiClient.createAccount(input);
	}

	/**
	 * Find account by ID
	 */
	async findById(id: string): Promise<Account | null> {
		try {
			return await apiClient.getAccountById(id);
		} catch (_error) {
			// If account not found, return null instead of throwing
			return null;
		}
	}

	/**
	 * Find accounts by user ID
	 */
	async findByUserId(
		userId: string,
		options: QueryOptions = {},
	): Promise<Account[]> {
		const filters: AccountFilters = {
			user_id: userId,
			is_active: true,
			limit: options.limit,
			offset: options.offset,
			sort_by: options.sort?.field,
			sort_direction: options.sort?.direction?.toUpperCase() as "ASC" | "DESC",
		};

		return await apiClient.getAccounts(filters);
	}

	/**
	 * Find accounts with type information
	 */
	async findWithType(
		userId: string,
		options: QueryOptions = {},
	): Promise<Account[]> {
		const filters = {
			limit: options.limit,
			offset: options.offset,
			sort_by: options.sort?.field,
			sort_direction: options.sort?.direction?.toUpperCase() as "ASC" | "DESC",
		};

		return await apiClient.getAccountsWithType(userId, filters);
	}

	/**
	 * Update an account
	 */
	async update(
		id: string,
		userId: string,
		input: Partial<UpdateAccountRequest>,
	): Promise<Account> {
		const updateRequest: UpdateAccountRequest = {
			...input,
		};
		return await apiClient.updateAccount(id, userId, updateRequest);
	}

	/**
	 * Delete an account (soft delete)
	 */
	async delete(id: string, userId: string): Promise<boolean> {
		return await apiClient.deleteAccount(id, userId);
	}

	/**
	 * Get total assets for a user
	 */
	async getTotalAssets(userId: string): Promise<number> {
		return await apiClient.getTotalAssets(userId);
	}

	/**
	 * Get total liabilities for a user
	 */
	async getTotalLiabilities(userId: string): Promise<number> {
		return await apiClient.getTotalLiabilities(userId);
	}

	/**
	 * Get account balances summary
	 */
	async getAccountBalances(userId: string): Promise<
		Array<{
			account_id: string;
			account_name: string;
			current_balance: number;
			currency: string;
		}>
	> {
		return await apiClient.getAccountBalances(userId);
	}
}

/**
 * Transaction repository using secure API calls
 */
export class ApiTransactionRepository {
	/**
	 * Create a new transaction
	 */
	async create(input: CreateTransactionRequest): Promise<Transaction> {
		return await apiClient.createTransaction(input);
	}

	/**
	 * Create a transaction with balance update
	 */
	async createWithBalanceUpdate(
		input: CreateTransactionRequest,
	): Promise<Transaction> {
		return await apiClient.createTransactionWithBalanceUpdate(input);
	}

	/**
	 * Find transaction by ID
	 */
	async findById(id: string): Promise<Transaction | null> {
		try {
			return await apiClient.getTransactionById(id);
		} catch (_error) {
			// If transaction not found, return null instead of throwing
			return null;
		}
	}

	/**
	 * Find transactions with details (account and category information)
	 */
	async findWithDetails(
		userId: string,
		filters?: Partial<TransactionFilters>,
		options?: QueryOptions,
	): Promise<QueryResult<Transaction>> {
		const apiOptions = {
			limit: options?.limit,
			offset: options?.offset,
			sort_by: options?.sort?.field,
			sort_direction: options?.sort?.direction?.toUpperCase() as "ASC" | "DESC",
		};

		return await apiClient.getTransactionsWithDetails(
			userId,
			filters,
			apiOptions,
		);
	}

	/**
	 * Get recent transactions
	 */
	async getRecent(userId: string, limit: number = 10): Promise<Transaction[]> {
		return await apiClient.getRecentTransactions(userId, limit);
	}

	/**
	 * Get category spending summary
	 */
	async getCategorySpending(
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<
		Array<{
			category_id: string;
			category_name: string;
			total_spent: number;
			transaction_count: number;
		}>
	> {
		return await apiClient.getCategorySpending(userId, startDate, endDate);
	}

	/**
	 * Update a transaction
	 */
	async update(
		id: string,
		userId: string,
		input: Partial<UpdateTransactionRequest>,
	): Promise<Transaction> {
		const updateRequest: UpdateTransactionRequest = {
			...input,
		};
		return await apiClient.updateTransaction(id, userId, updateRequest);
	}

	/**
	 * Delete a transaction
	 */
	async delete(id: string, userId: string): Promise<boolean> {
		return await apiClient.deleteTransaction(id, userId);
	}
}

/**
 * Main API service class that replaces the database service
 */
export class ApiService {
	public accounts = new ApiAccountRepository();
	public transactions = new ApiTransactionRepository();

	/**
	 * Initialize the API service
	 * This replaces the database service initialization
	 */
	async initialize(): Promise<void> {
		try {
			// No database connection needed - just verify API is available
			// This could include checking authentication status or API health
			console.log("API service initialized successfully");
		} catch (error) {
			console.error("Failed to initialize API service:", error);
			throw error;
		}
	}

	/**
	 * Get comprehensive dashboard summary
	 */
	async getDashboardSummary(userId: string): Promise<DashboardSummary> {
		return await apiClient.getDashboardSummary(userId);
	}

	/**
	 * Close/cleanup the service
	 */
	async close(): Promise<void> {
		// No cleanup needed for API service
		console.log("API service closed");
	}
}

// Export singleton API service instance
export const apiService = new ApiService();

// Utility functions for compatibility
export const apiUtils = {
	/**
	 * Generate a new UUID for database records
	 */
	generateId: () => apiClient.generateId(),

	/**
	 * Format a date for API storage
	 */
	formatDate: (date: Date | string) => apiClient.formatDateForDb(date),

	/**
	 * Format a datetime for API storage
	 */
	formatDateTime: (date: Date | string) => apiClient.formatDateTimeForDb(date),

	/**
	 * Validate that a string is a valid UUID
	 */
	isValidId: (id: string) => apiClient.isValidId(id),

	/**
	 * Sanitize a string for search queries
	 */
	sanitizeForLike: (input: string) => apiClient.sanitizeForLike(input),

	/**
	 * Get date range for common periods
	 */
	getDateRange: (period: "today" | "week" | "month" | "quarter" | "year") =>
		apiClient.getDateRange(period),
};

// Export everything for convenience
export default {
	service: apiService,
	repositories: {
		accounts: apiService.accounts,
		transactions: apiService.transactions,
	},
	utils: apiUtils,
};
