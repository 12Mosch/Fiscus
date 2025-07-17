/**
 * Centralized API client for Fiscus application
 * Provides type-safe access to all Tauri commands with proper error handling
 */

import { invoke } from "@tauri-apps/api/core";
import type {
	Account,
	// Filter types
	AccountFilters,
	AccountSummaryResponse,
	ApiError,
	Budget,
	BudgetFilters,
	BudgetPeriod,
	BudgetSummaryResponse,
	BulkTransactionRequest,
	Category,
	CategoryFilters,
	ChangePasswordRequest,
	CreateAccountRequest,
	CreateBudgetPeriodRequest,
	CreateBudgetRequest,
	CreateCategoryRequest,
	CreateGoalRequest,
	CreateTransactionRequest,
	CreateTransferRequest,
	// Request types
	CreateUserRequest,
	Goal,
	GoalFilters,
	LoginRequest,
	// Response types
	LoginResponse,
	PaginatedResponse,
	ReportData,
	Transaction,
	TransactionFilters,
	TransactionStatsResponse,
	TransactionSummaryResponse,
	Transfer,
	UpdateAccountRequest,
	UpdateBudgetRequest,
	UpdateCategoryRequest,
	UpdateGoalRequest,
	UpdateTransactionRequest,
	// Entity types
	User,
} from "../types/api";

/**
 * Custom error class for API operations
 */
export class FiscusApiError extends Error {
	constructor(
		message: string,
		public code: string,
		public statusCode?: number,
	) {
		super(message);
		this.name = "FiscusApiError";
	}
}

/**
 * Handle API errors from Tauri commands
 */
function handleApiError(error: unknown): FiscusApiError {
	if (
		typeof error === "object" &&
		error !== null &&
		"type" in error &&
		"message" in error
	) {
		const apiError = error as ApiError;
		return new FiscusApiError(apiError.message, apiError.type);
	}

	if (typeof error === "string") {
		return new FiscusApiError(error, "UNKNOWN_ERROR");
	}

	return new FiscusApiError("An unexpected error occurred", "UNKNOWN_ERROR");
}

/**
 * Centralized API client class
 */
export class FiscusApiClient {
	// ============================================================================
	// Authentication Methods
	// ============================================================================

	/**
	 * Create a new user account
	 * @param request User creation data
	 * @returns Promise resolving to user information
	 */
	async createUser(request: CreateUserRequest): Promise<User> {
		try {
			return await invoke("create_user", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Authenticate user login
	 * @param request Login credentials
	 * @returns Promise resolving to login response with user info
	 */
	async loginUser(request: LoginRequest): Promise<LoginResponse> {
		try {
			return await invoke("login_user", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Change user password
	 * @param request Password change data
	 * @returns Promise resolving to success status
	 */
	async changePassword(request: ChangePasswordRequest): Promise<boolean> {
		try {
			return await invoke("change_password", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get current user information
	 * @param userId User ID
	 * @returns Promise resolving to user information
	 */
	async getCurrentUser(userId: string): Promise<User> {
		try {
			return await invoke("get_current_user", { userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Account Methods
	// ============================================================================

	/**
	 * Create a new account
	 * @param request Account creation data
	 * @returns Promise resolving to created account
	 */
	async createAccount(request: CreateAccountRequest): Promise<Account> {
		try {
			return await invoke("create_account", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get accounts with optional filtering
	 * @param filters Account filter criteria
	 * @returns Promise resolving to array of accounts
	 */
	async getAccounts(filters: AccountFilters): Promise<Account[]> {
		try {
			return await invoke("get_accounts", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a single account by ID
	 * @param accountId Account ID
	 * @returns Promise resolving to account
	 */
	async getAccountById(accountId: string): Promise<Account> {
		try {
			return await invoke("get_account_by_id", { accountId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update an account
	 * @param accountId Account ID
	 * @param userId User ID
	 * @param request Update data
	 * @returns Promise resolving to updated account
	 */
	async updateAccount(
		accountId: string,
		userId: string,
		request: UpdateAccountRequest,
	): Promise<Account> {
		try {
			return await invoke("update_account", { accountId, userId, request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Delete an account
	 * @param accountId Account ID
	 * @param userId User ID
	 * @returns Promise resolving to success status
	 */
	async deleteAccount(accountId: string, userId: string): Promise<boolean> {
		try {
			return await invoke("delete_account", { accountId, userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get account summary for a user
	 * @param userId User ID
	 * @returns Promise resolving to account summary
	 */
	async getAccountSummary(userId: string): Promise<AccountSummaryResponse> {
		try {
			return await invoke("get_account_summary", { userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Transaction Methods
	// ============================================================================

	/**
	 * Create a new transaction
	 * @param request Transaction creation data
	 * @returns Promise resolving to created transaction
	 */
	async createTransaction(
		request: CreateTransactionRequest,
	): Promise<Transaction> {
		try {
			return await invoke("create_transaction", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get transactions with filtering and pagination
	 * @param filters Transaction filter criteria
	 * @returns Promise resolving to array of transactions
	 */
	async getTransactions(filters: TransactionFilters): Promise<Transaction[]> {
		try {
			return await invoke("get_transactions", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a single transaction by ID
	 * @param transactionId Transaction ID
	 * @returns Promise resolving to transaction
	 */
	async getTransactionById(transactionId: string): Promise<Transaction> {
		try {
			return await invoke("get_transaction_by_id", { transactionId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update a transaction
	 * @param transactionId Transaction ID
	 * @param userId User ID
	 * @param request Update data
	 * @returns Promise resolving to updated transaction
	 */
	async updateTransaction(
		transactionId: string,
		userId: string,
		request: UpdateTransactionRequest,
	): Promise<Transaction> {
		try {
			return await invoke("update_transaction", {
				transactionId,
				userId,
				request,
			});
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Delete a transaction
	 * @param transactionId Transaction ID
	 * @param userId User ID
	 * @returns Promise resolving to success status
	 */
	async deleteTransaction(
		transactionId: string,
		userId: string,
	): Promise<boolean> {
		try {
			return await invoke("delete_transaction", { transactionId, userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Create a transfer between accounts
	 * @param request Transfer creation data
	 * @returns Promise resolving to created transfer
	 */
	async createTransfer(request: CreateTransferRequest): Promise<Transfer> {
		try {
			return await invoke("create_transfer", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a transfer by ID
	 * @param transferId Transfer ID
	 * @returns Promise resolving to transfer
	 */
	async getTransferById(transferId: string): Promise<Transfer> {
		try {
			return await invoke("get_transfer_by_id", { transferId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get transaction summary for a user
	 * @param userId User ID
	 * @param startDate Optional start date filter
	 * @param endDate Optional end date filter
	 * @returns Promise resolving to transaction summary
	 */
	async getTransactionSummary(
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<TransactionSummaryResponse> {
		try {
			return await invoke("get_transaction_summary", {
				userId,
				startDate,
				endDate,
			});
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get transactions with pagination support
	 * @param filters Transaction filter criteria
	 * @returns Promise resolving to paginated transactions
	 */
	async getTransactionsPaginated(
		filters: TransactionFilters,
	): Promise<PaginatedResponse<Transaction>> {
		try {
			return await invoke("get_transactions_paginated", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get transaction statistics
	 * @param filters Transaction filter criteria
	 * @returns Promise resolving to transaction statistics
	 */
	async getTransactionStats(
		filters: TransactionFilters,
	): Promise<TransactionStatsResponse> {
		try {
			return await invoke("get_transaction_stats", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Perform bulk operations on transactions
	 * @param request Bulk operation request
	 * @returns Promise resolving to operation result message
	 */
	async bulkTransactionOperations(
		request: BulkTransactionRequest,
	): Promise<string> {
		try {
			return await invoke("bulk_transaction_operations", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Category Methods
	// ============================================================================

	/**
	 * Create a new category
	 * @param request Category creation data
	 * @returns Promise resolving to created category
	 */
	async createCategory(request: CreateCategoryRequest): Promise<Category> {
		try {
			return await invoke("create_category", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get categories with optional filtering
	 * @param filters Category filter criteria
	 * @returns Promise resolving to array of categories
	 */
	async getCategories(filters: CategoryFilters): Promise<Category[]> {
		try {
			return await invoke("get_categories", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a single category by ID
	 * @param categoryId Category ID
	 * @returns Promise resolving to category
	 */
	async getCategoryById(categoryId: string): Promise<Category> {
		try {
			return await invoke("get_category_by_id", { categoryId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update a category
	 * @param categoryId Category ID
	 * @param userId User ID
	 * @param request Update data
	 * @returns Promise resolving to updated category
	 */
	async updateCategory(
		categoryId: string,
		userId: string,
		request: UpdateCategoryRequest,
	): Promise<Category> {
		try {
			return await invoke("update_category", { categoryId, userId, request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Delete a category
	 * @param categoryId Category ID
	 * @param userId User ID
	 * @returns Promise resolving to success status
	 */
	async deleteCategory(categoryId: string, userId: string): Promise<boolean> {
		try {
			return await invoke("delete_category", { categoryId, userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get category hierarchy (tree structure)
	 * @param userId User ID
	 * @param isIncome Optional filter for income categories
	 * @returns Promise resolving to array of categories in hierarchy
	 */
	async getCategoryHierarchy(
		userId: string,
		isIncome?: boolean,
	): Promise<Category[]> {
		try {
			return await invoke("get_category_hierarchy", { userId, isIncome });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Budget Methods
	// ============================================================================

	/**
	 * Create a new budget period
	 * @param request Budget period creation data
	 * @returns Promise resolving to created budget period
	 */
	async createBudgetPeriod(
		request: CreateBudgetPeriodRequest,
	): Promise<BudgetPeriod> {
		try {
			return await invoke("create_budget_period", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get budget periods for a user
	 * @param userId User ID
	 * @param isActive Optional filter for active periods
	 * @returns Promise resolving to array of budget periods
	 */
	async getBudgetPeriods(
		userId: string,
		isActive?: boolean,
	): Promise<BudgetPeriod[]> {
		try {
			return await invoke("get_budget_periods", { userId, isActive });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a budget period by ID
	 * @param periodId Budget period ID
	 * @returns Promise resolving to budget period
	 */
	async getBudgetPeriodById(periodId: string): Promise<BudgetPeriod> {
		try {
			return await invoke("get_budget_period_by_id", { periodId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Create a new budget
	 * @param request Budget creation data
	 * @returns Promise resolving to created budget
	 */
	async createBudget(request: CreateBudgetRequest): Promise<Budget> {
		try {
			return await invoke("create_budget", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get budgets with filtering
	 * @param filters Budget filter criteria
	 * @returns Promise resolving to array of budgets
	 */
	async getBudgets(filters: BudgetFilters): Promise<Budget[]> {
		try {
			return await invoke("get_budgets", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a single budget by ID
	 * @param budgetId Budget ID
	 * @returns Promise resolving to budget
	 */
	async getBudgetById(budgetId: string): Promise<Budget> {
		try {
			return await invoke("get_budget_by_id", { budgetId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update a budget
	 * @param budgetId Budget ID
	 * @param userId User ID
	 * @param request Update data
	 * @returns Promise resolving to updated budget
	 */
	async updateBudget(
		budgetId: string,
		userId: string,
		request: UpdateBudgetRequest,
	): Promise<Budget> {
		try {
			return await invoke("update_budget", { budgetId, userId, request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Delete a budget
	 * @param budgetId Budget ID
	 * @param userId User ID
	 * @returns Promise resolving to success status
	 */
	async deleteBudget(budgetId: string, userId: string): Promise<boolean> {
		try {
			return await invoke("delete_budget", { budgetId, userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get budget summary for a user and period
	 * @param userId User ID
	 * @param budgetPeriodId Optional budget period ID
	 * @returns Promise resolving to budget summary
	 */
	async getBudgetSummary(
		userId: string,
		budgetPeriodId?: string,
	): Promise<BudgetSummaryResponse> {
		try {
			return await invoke("get_budget_summary", { userId, budgetPeriodId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Goal Methods
	// ============================================================================

	/**
	 * Create a new goal
	 * @param request Goal creation data
	 * @returns Promise resolving to created goal
	 */
	async createGoal(request: CreateGoalRequest): Promise<Goal> {
		try {
			return await invoke("create_goal", { request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get goals with filtering
	 * @param filters Goal filter criteria
	 * @returns Promise resolving to array of goals
	 */
	async getGoals(filters: GoalFilters): Promise<Goal[]> {
		try {
			return await invoke("get_goals", { filters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get a single goal by ID
	 * @param goalId Goal ID
	 * @returns Promise resolving to goal
	 */
	async getGoalById(goalId: string): Promise<Goal> {
		try {
			return await invoke("get_goal_by_id", { goalId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update a goal
	 * @param goalId Goal ID
	 * @param userId User ID
	 * @param request Update data
	 * @returns Promise resolving to updated goal
	 */
	async updateGoal(
		goalId: string,
		userId: string,
		request: UpdateGoalRequest,
	): Promise<Goal> {
		try {
			return await invoke("update_goal", { goalId, userId, request });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Delete a goal
	 * @param goalId Goal ID
	 * @param userId User ID
	 * @returns Promise resolving to success status
	 */
	async deleteGoal(goalId: string, userId: string): Promise<boolean> {
		try {
			return await invoke("delete_goal", { goalId, userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Update goal progress (add to current amount)
	 * @param goalId Goal ID
	 * @param userId User ID
	 * @param amount Amount to add to progress
	 * @returns Promise resolving to updated goal
	 */
	async updateGoalProgress(
		goalId: string,
		userId: string,
		amount: number,
	): Promise<Goal> {
		try {
			return await invoke("update_goal_progress", { goalId, userId, amount });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get goal progress summary for a user
	 * @param userId User ID
	 * @returns Promise resolving to goal progress summary
	 */
	async getGoalProgressSummary(userId: string): Promise<ReportData> {
		try {
			return await invoke("get_goal_progress_summary", { userId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Report Methods
	// ============================================================================

	/**
	 * Get financial overview report for a user
	 * @param userId User ID
	 * @param startDate Optional start date filter
	 * @param endDate Optional end date filter
	 * @returns Promise resolving to financial overview data
	 */
	async getFinancialOverview(
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<ReportData> {
		try {
			return await invoke("get_financial_overview", {
				userId,
				startDate,
				endDate,
			});
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get spending by category report
	 * @param userId User ID
	 * @param startDate Optional start date filter
	 * @param endDate Optional end date filter
	 * @param limit Optional limit for results
	 * @returns Promise resolving to spending by category data
	 */
	async getSpendingByCategory(
		userId: string,
		startDate?: string,
		endDate?: string,
		limit?: number,
	): Promise<ReportData[]> {
		try {
			return await invoke("get_spending_by_category", {
				userId,
				startDate,
				endDate,
				limit,
			});
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get monthly spending trend
	 * @param userId User ID
	 * @param months Optional number of months to include
	 * @returns Promise resolving to monthly spending trend data
	 */
	async getMonthlySpendingTrend(
		userId: string,
		months?: number,
	): Promise<ReportData[]> {
		try {
			return await invoke("get_monthly_spending_trend", { userId, months });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get account balance history
	 * @param userId User ID
	 * @param accountId Optional account ID filter
	 * @param days Optional number of days to include
	 * @returns Promise resolving to account balance history data
	 */
	async getAccountBalanceHistory(
		userId: string,
		accountId?: string,
		days?: number,
	): Promise<ReportData[]> {
		try {
			return await invoke("get_account_balance_history", {
				userId,
				accountId,
				days,
			});
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get budget performance report
	 * @param userId User ID
	 * @param budgetPeriodId Optional budget period ID
	 * @returns Promise resolving to budget performance data
	 */
	async getBudgetPerformance(
		userId: string,
		budgetPeriodId?: string,
	): Promise<ReportData[]> {
		try {
			return await invoke("get_budget_performance", { userId, budgetPeriodId });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get net worth progression over time
	 * @param userId User ID
	 * @param months Optional number of months to include
	 * @returns Promise resolving to net worth progression data
	 */
	async getNetWorthProgression(
		userId: string,
		months?: number,
	): Promise<ReportData[]> {
		try {
			return await invoke("get_net_worth_progression", { userId, months });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Enhanced Account Methods (for database service compatibility)
	// ============================================================================

	/**
	 * Get accounts with their account type information
	 * @param userId User ID
	 * @param filters Optional account filters
	 * @returns Promise resolving to accounts with type details
	 */
	async getAccountsWithType(
		userId: string,
		filters?: Omit<AccountFilters, "user_id">,
	): Promise<Account[]> {
		try {
			const accountFilters: AccountFilters = { user_id: userId, ...filters };
			// The backend should return accounts with account_type joined
			return await invoke("get_accounts", { filters: accountFilters });
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get total assets for a user
	 * @param userId User ID
	 * @returns Promise resolving to total assets amount
	 */
	async getTotalAssets(userId: string): Promise<number> {
		try {
			// This will need to be implemented as a new Tauri command
			// For now, we'll calculate it from account summary
			const summary = await this.getAccountSummary(userId);
			return summary.total_assets || 0;
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get total liabilities for a user
	 * @param userId User ID
	 * @returns Promise resolving to total liabilities amount
	 */
	async getTotalLiabilities(userId: string): Promise<number> {
		try {
			// This will need to be implemented as a new Tauri command
			// For now, we'll calculate it from account summary
			const summary = await this.getAccountSummary(userId);
			return summary.total_liabilities || 0;
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get account balances summary for a user
	 * @param userId User ID
	 * @returns Promise resolving to account balances
	 */
	async getAccountBalances(userId: string): Promise<
		Array<{
			account_id: string;
			account_name: string;
			current_balance: number;
			currency: string;
		}>
	> {
		try {
			const accounts = await this.getAccounts({
				user_id: userId,
				is_active: true,
			});
			return accounts.map((account) => ({
				account_id: account.id,
				account_name: account.name,
				current_balance: account.balance,
				currency: account.currency,
			}));
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Enhanced Transaction Methods (for database service compatibility)
	// ============================================================================

	/**
	 * Get transactions with detailed information (account and category details)
	 * @param userId User ID
	 * @param filters Optional transaction filters
	 * @param options Query options for pagination and sorting
	 * @returns Promise resolving to transactions with details and pagination info
	 */
	async getTransactionsWithDetails(
		userId: string,
		filters?: Omit<TransactionFilters, "user_id">,
		options?: {
			limit?: number;
			offset?: number;
			sort_by?: string;
			sort_direction?: "ASC" | "DESC";
		},
	): Promise<{
		data: Transaction[];
		total: number;
		page: number;
		limit: number;
	}> {
		try {
			const transactionFilters: TransactionFilters = {
				user_id: userId,
				...filters,
				...options,
			};

			// Get transactions - the backend should return with joined account/category data
			const transactions = await this.getTransactions(transactionFilters);

			// For now, return in the expected format
			// TODO: Backend should return pagination info
			return {
				data: transactions,
				total: transactions.length,
				page: Math.floor((options?.offset || 0) / (options?.limit || 50)) + 1,
				limit: options?.limit || 50,
			};
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get recent transactions for a user
	 * @param userId User ID
	 * @param limit Number of transactions to retrieve
	 * @returns Promise resolving to recent transactions
	 */
	async getRecentTransactions(
		userId: string,
		limit: number = 10,
	): Promise<Transaction[]> {
		try {
			const filters: TransactionFilters = {
				user_id: userId,
				limit,
				sort_by: "transaction_date",
				sort_direction: "DESC",
			};
			return await this.getTransactions(filters);
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Get category spending summary for a user
	 * @param userId User ID
	 * @param startDate Optional start date filter
	 * @param endDate Optional end date filter
	 * @returns Promise resolving to category spending data
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
		try {
			// This should be implemented as a dedicated Tauri command for better performance
			// For now, we'll use the existing report methods
			const reportData = await this.getSpendingByCategory(
				userId,
				startDate,
				endDate,
			);

			// Transform report data to expected format
			if (Array.isArray(reportData)) {
				return reportData.map((item: Record<string, unknown>) => ({
					category_id: (item.category_id as string) || "",
					category_name:
						(item.category as string) || (item.name as string) || "Unknown",
					total_spent: Math.abs(
						(item.amount as number) || (item.total as number) || 0,
					),
					transaction_count:
						(item.count as number) || (item.transaction_count as number) || 0,
				}));
			}

			return [];
		} catch (error) {
			throw handleApiError(error);
		}
	}

	/**
	 * Create a transaction with automatic balance update
	 * @param request Transaction creation data
	 * @returns Promise resolving to created transaction
	 */
	async createTransactionWithBalanceUpdate(
		request: CreateTransactionRequest,
	): Promise<Transaction> {
		try {
			// The backend should handle balance updates automatically
			// This is just an alias to the regular create method
			return await this.createTransaction(request);
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Dashboard and Summary Methods
	// ============================================================================

	/**
	 * Get comprehensive dashboard summary for a user
	 * @param userId User ID
	 * @returns Promise resolving to dashboard summary data
	 */
	async getDashboardSummary(userId: string): Promise<{
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
	}> {
		try {
			// Fetch all dashboard data in parallel for better performance
			const [
				totalAssets,
				totalLiabilities,
				accountBalances,
				recentTransactions,
				categorySpending,
				transactionSummary,
			] = await Promise.all([
				this.getTotalAssets(userId),
				this.getTotalLiabilities(userId),
				this.getAccountBalances(userId),
				this.getRecentTransactions(userId, 10),
				this.getCategorySpending(userId),
				this.getTransactionSummary(userId),
			]);

			const netWorth = totalAssets - totalLiabilities;

			return {
				total_assets: totalAssets,
				total_liabilities: totalLiabilities,
				net_worth: netWorth,
				monthly_income: transactionSummary.total_income || 0,
				monthly_expenses: Math.abs(transactionSummary.total_expenses || 0),
				recent_transactions: recentTransactions,
				account_balances: accountBalances,
				top_categories: categorySpending.slice(0, 5), // Top 5 categories
			};
		} catch (error) {
			throw handleApiError(error);
		}
	}

	// ============================================================================
	// Utility Methods
	// ============================================================================

	/**
	 * Generate a new UUID for database records
	 * @returns Promise resolving to a new UUID string
	 */
	async generateId(): Promise<string> {
		// For security, we should generate IDs on the backend
		// For now, use crypto.randomUUID() if available
		if (typeof crypto !== "undefined" && crypto.randomUUID) {
			return crypto.randomUUID();
		}

		// Fallback: generate a simple UUID-like string
		// In production, this should be handled by the backend
		return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
			const r = (Math.random() * 16) | 0;
			const v = c === "x" ? r : (r & 0x3) | 0x8;
			return v.toString(16);
		});
	}

	/**
	 * Format a date for database storage
	 * @param date Date to format
	 * @returns Formatted date string (YYYY-MM-DD)
	 */
	formatDateForDb(date: Date | string): string {
		const dateObj = typeof date === "string" ? new Date(date) : date;

		if (Number.isNaN(dateObj.getTime())) {
			throw new FiscusApiError("Invalid date provided", "INVALID_INPUT");
		}

		return dateObj.toISOString().split("T")[0];
	}

	/**
	 * Format a datetime for database storage
	 * @param date Date to format
	 * @returns Formatted datetime string (ISO format)
	 */
	formatDateTimeForDb(date: Date | string): string {
		const dateObj = typeof date === "string" ? new Date(date) : date;

		if (Number.isNaN(dateObj.getTime())) {
			throw new FiscusApiError("Invalid date provided", "INVALID_INPUT");
		}

		return dateObj.toISOString();
	}

	/**
	 * Validate that a string is a valid UUID
	 * @param id String to validate
	 * @returns True if valid UUID, false otherwise
	 */
	isValidId(id: string): boolean {
		const uuidRegex =
			/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
		return uuidRegex.test(id);
	}

	/**
	 * Sanitize a string for SQL LIKE queries
	 * @param input String to sanitize
	 * @returns Sanitized string
	 */
	sanitizeForLike(input: string): string {
		return input.replace(/[%_]/g, "\\$&");
	}

	/**
	 * Get date range for common periods
	 * @param period Period type
	 * @returns Object with start and end dates
	 */
	getDateRange(period: "today" | "week" | "month" | "quarter" | "year"): {
		start: string;
		end: string;
	} {
		const now = new Date();
		const start = new Date();

		switch (period) {
			case "today":
				start.setHours(0, 0, 0, 0);
				break;
			case "week":
				start.setDate(now.getDate() - now.getDay());
				start.setHours(0, 0, 0, 0);
				break;
			case "month":
				start.setDate(1);
				start.setHours(0, 0, 0, 0);
				break;
			case "quarter": {
				const quarter = Math.floor(now.getMonth() / 3);
				start.setMonth(quarter * 3, 1);
				start.setHours(0, 0, 0, 0);
				break;
			}
			case "year":
				start.setMonth(0, 1);
				start.setHours(0, 0, 0, 0);
				break;
		}

		return {
			start: this.formatDateForDb(start),
			end: this.formatDateForDb(now),
		};
	}
}

/**
 * Singleton instance of the API client
 */
export const apiClient = new FiscusApiClient();
