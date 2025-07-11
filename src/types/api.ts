/**
 * TypeScript type definitions for Fiscus API
 * These types mirror the Rust command signatures exactly
 */

// ============================================================================
// Base Types and Enums
// ============================================================================

/**
 * Transaction type enumeration
 */
export type TransactionType = "income" | "expense" | "transfer";

/**
 * Transaction status enumeration
 */
export type TransactionStatus = "pending" | "completed" | "cancelled";

/**
 * Goal status enumeration
 */
export type GoalStatus = "active" | "completed" | "paused" | "cancelled";

/**
 * Sort direction for queries
 */
export type SortDirection = "ASC" | "DESC";

// ============================================================================
// Entity Types
// ============================================================================

/**
 * User entity
 */
export interface User {
	id: string;
	username: string;
	email?: string;
	created_at: string;
	updated_at: string;
}

/**
 * Account Type entity
 */
export interface AccountType {
	id: string;
	name: string;
	description?: string;
	is_asset: boolean;
	created_at: string;
}

/**
 * Account entity
 */
export interface Account {
	id: string;
	user_id: string;
	account_type_id: string;
	name: string;
	balance: number;
	currency: string;
	account_number?: string;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

/**
 * Category entity
 */
export interface Category {
	id: string;
	user_id: string;
	name: string;
	description?: string;
	color?: string;
	icon?: string;
	parent_category_id?: string;
	is_income: boolean;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

/**
 * Transaction entity
 */
export interface Transaction {
	id: string;
	user_id: string;
	account_id: string;
	category_id?: string;
	amount: number;
	description: string;
	notes?: string;
	transaction_date: string;
	transaction_type: TransactionType;
	status: TransactionStatus;
	reference_number?: string;
	payee?: string;
	tags?: string[];
	created_at: string;
	updated_at: string;
}

/**
 * Budget Period entity
 */
export interface BudgetPeriod {
	id: string;
	user_id: string;
	name: string;
	start_date: string;
	end_date: string;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

/**
 * Budget entity
 */
export interface Budget {
	id: string;
	user_id: string;
	budget_period_id: string;
	category_id: string;
	allocated_amount: number;
	spent_amount: number;
	notes?: string;
	created_at: string;
	updated_at: string;
}

/**
 * Goal entity
 */
export interface Goal {
	id: string;
	user_id: string;
	name: string;
	description?: string;
	target_amount: number;
	current_amount: number;
	target_date?: string;
	priority: number;
	status: GoalStatus;
	category?: string;
	created_at: string;
	updated_at: string;
}

/**
 * Transfer entity
 */
export interface Transfer {
	id: string;
	user_id: string;
	from_account_id: string;
	to_account_id: string;
	amount: number;
	description: string;
	transfer_date: string;
	status: TransactionStatus;
	from_transaction_id: string;
	to_transaction_id: string;
	created_at: string;
	updated_at: string;
}

// ============================================================================
// Request DTOs
// ============================================================================

/**
 * Create user request
 */
export interface CreateUserRequest {
	/** Username (3-50 characters) */
	username: string;
	/** Optional email address */
	email?: string;
	/** Password (8-128 characters) */
	password: string;
}

/**
 * Login request
 */
export interface LoginRequest {
	/** Username */
	username: string;
	/** Password */
	password: string;
}

/**
 * Change password request
 */
export interface ChangePasswordRequest {
	/** User ID */
	user_id: string;
	/** Current password */
	current_password: string;
	/** New password (8-128 characters) */
	new_password: string;
}

/**
 * Create account request
 */
export interface CreateAccountRequest {
	/** User ID */
	user_id: string;
	/** Account type ID */
	account_type_id: string;
	/** Account name (1-100 characters) */
	name: string;
	/** Initial balance (optional, defaults to 0) */
	balance?: number;
	/** Currency code (3 characters, e.g., 'USD') */
	currency: string;
	/** Optional account number */
	account_number?: string;
}

/**
 * Update account request
 */
export interface UpdateAccountRequest {
	/** New account name */
	name?: string;
	/** New balance */
	balance?: number;
	/** New account number */
	account_number?: string;
	/** Active status */
	is_active?: boolean;
}

/**
 * Create category request
 */
export interface CreateCategoryRequest {
	/** User ID */
	user_id: string;
	/** Category name (1-100 characters) */
	name: string;
	/** Optional description (0-500 characters) */
	description?: string;
	/** Optional color (hex code) */
	color?: string;
	/** Optional icon identifier */
	icon?: string;
	/** Optional parent category ID */
	parent_category_id?: string;
	/** Whether this is an income category */
	is_income: boolean;
}

/**
 * Update category request
 */
export interface UpdateCategoryRequest {
	/** New category name */
	name?: string;
	/** New description */
	description?: string;
	/** New color */
	color?: string;
	/** New icon */
	icon?: string;
	/** New parent category ID (empty string to remove parent) */
	parent_category_id?: string;
	/** Active status */
	is_active?: boolean;
}

/**
 * Create transaction request
 */
export interface CreateTransactionRequest {
	/** User ID */
	user_id: string;
	/** Account ID */
	account_id: string;
	/** Optional category ID */
	category_id?: string;
	/** Transaction amount */
	amount: number;
	/** Transaction description (1-255 characters) */
	description: string;
	/** Optional notes */
	notes?: string;
	/** Transaction date (ISO 8601 format) */
	transaction_date: string;
	/** Transaction type */
	transaction_type: TransactionType;
	/** Optional reference number */
	reference_number?: string;
	/** Optional payee */
	payee?: string;
	/** Optional tags */
	tags?: string[];
}

/**
 * Update transaction request
 */
export interface UpdateTransactionRequest {
	/** New category ID */
	category_id?: string;
	/** New amount */
	amount?: number;
	/** New description */
	description?: string;
	/** New notes */
	notes?: string;
	/** New transaction date */
	transaction_date?: string;
	/** New transaction type */
	transaction_type?: TransactionType;
	/** New status */
	status?: TransactionStatus;
	/** New reference number */
	reference_number?: string;
	/** New payee */
	payee?: string;
	/** New tags */
	tags?: string[];
}

/**
 * Create transfer request
 */
export interface CreateTransferRequest {
	/** User ID */
	user_id: string;
	/** Source account ID */
	from_account_id: string;
	/** Destination account ID */
	to_account_id: string;
	/** Transfer amount (must be positive) */
	amount: number;
	/** Transfer description (1-255 characters) */
	description: string;
	/** Transfer date (ISO 8601 format) */
	transfer_date: string;
}

/**
 * Create budget period request
 */
export interface CreateBudgetPeriodRequest {
	/** User ID */
	user_id: string;
	/** Period name (1-100 characters) */
	name: string;
	/** Start date (YYYY-MM-DD format) */
	start_date: string;
	/** End date (YYYY-MM-DD format) */
	end_date: string;
}

/**
 * Create budget request
 */
export interface CreateBudgetRequest {
	/** User ID */
	user_id: string;
	/** Budget period ID */
	budget_period_id: string;
	/** Category ID */
	category_id: string;
	/** Allocated amount (must be positive) */
	allocated_amount: number;
	/** Optional notes */
	notes?: string;
}

/**
 * Update budget request
 */
export interface UpdateBudgetRequest {
	/** New allocated amount */
	allocated_amount?: number;
	/** New notes */
	notes?: string;
}

/**
 * Create goal request
 */
export interface CreateGoalRequest {
	/** User ID */
	user_id: string;
	/** Goal name (1-100 characters) */
	name: string;
	/** Optional description (0-500 characters) */
	description?: string;
	/** Target amount (must be positive) */
	target_amount: number;
	/** Optional target date (YYYY-MM-DD format) */
	target_date?: string;
	/** Optional priority (1-5, defaults to 1) */
	priority?: number;
	/** Optional category */
	category?: string;
}

/**
 * Update goal request
 */
export interface UpdateGoalRequest {
	/** New goal name */
	name?: string;
	/** New description */
	description?: string;
	/** New target amount */
	target_amount?: number;
	/** New current amount */
	current_amount?: number;
	/** New target date */
	target_date?: string;
	/** New priority */
	priority?: number;
	/** New status */
	status?: GoalStatus;
	/** New category */
	category?: string;
}

// ============================================================================
// Filter DTOs
// ============================================================================

/**
 * Account filters for querying
 */
export interface AccountFilters {
	/** User ID (required) */
	user_id: string;
	/** Filter by account type */
	account_type_id?: string;
	/** Filter by active status */
	is_active?: boolean;
	/** Sort field */
	sort_by?: string;
	/** Sort direction */
	sort_direction?: SortDirection;
	/** Limit results */
	limit?: number;
	/** Offset for pagination */
	offset?: number;
}

/**
 * Transaction filters for querying
 */
export interface TransactionFilters {
	/** User ID (required) */
	user_id: string;
	/** Filter by account */
	account_id?: string;
	/** Filter by category */
	category_id?: string;
	/** Filter by transaction type */
	transaction_type?: TransactionType;
	/** Filter by status */
	status?: TransactionStatus;
	/** Start date filter (YYYY-MM-DD) */
	start_date?: string;
	/** End date filter (YYYY-MM-DD) */
	end_date?: string;
	/** Minimum amount filter */
	min_amount?: number;
	/** Maximum amount filter */
	max_amount?: number;
	/** Search in description, payee, notes */
	search?: string;
	/** Sort field */
	sort_by?: string;
	/** Sort direction */
	sort_direction?: SortDirection;
	/** Limit results */
	limit?: number;
	/** Offset for pagination */
	offset?: number;
}

/**
 * Category filters for querying
 */
export interface CategoryFilters {
	/** User ID (required) */
	user_id: string;
	/** Filter by parent category (empty string for root categories) */
	parent_category_id?: string;
	/** Filter by income/expense type */
	is_income?: boolean;
	/** Filter by active status */
	is_active?: boolean;
	/** Sort field */
	sort_by?: string;
	/** Sort direction */
	sort_direction?: SortDirection;
}

/**
 * Budget filters for querying
 */
export interface BudgetFilters {
	/** User ID (required) */
	user_id: string;
	/** Filter by budget period */
	budget_period_id?: string;
	/** Filter by category */
	category_id?: string;
	/** Sort field */
	sort_by?: string;
	/** Sort direction */
	sort_direction?: SortDirection;
}

/**
 * Goal filters for querying
 */
export interface GoalFilters {
	/** User ID (required) */
	user_id: string;
	/** Filter by status */
	status?: GoalStatus;
	/** Filter by category */
	category?: string;
	/** Sort field */
	sort_by?: string;
	/** Sort direction */
	sort_direction?: SortDirection;
}

// ============================================================================
// Response DTOs
// ============================================================================

/**
 * Login response
 */
export interface LoginResponse {
	/** User information */
	user: User;
	/** Optional session token */
	session_token?: string;
}

/**
 * Paginated response wrapper
 */
export interface PaginatedResponse<T> {
	/** Data items */
	data: T[];
	/** Total number of items */
	total: number;
	/** Current page */
	page: number;
	/** Items per page */
	per_page: number;
	/** Total number of pages */
	total_pages: number;
}

/**
 * Account summary response
 */
export interface AccountSummaryResponse {
	/** Total assets value */
	total_assets: number;
	/** Total liabilities value */
	total_liabilities: number;
	/** Net worth (assets - liabilities) */
	net_worth: number;
	/** Number of accounts */
	account_count: number;
}

/**
 * Budget summary response
 */
export interface BudgetSummaryResponse {
	/** Total allocated amount */
	total_allocated: number;
	/** Total spent amount */
	total_spent: number;
	/** Remaining amount */
	remaining: number;
	/** Number of categories over budget */
	categories_over_budget: number;
	/** Number of categories under budget */
	categories_under_budget: number;
}

/**
 * Transaction summary response
 */
export interface TransactionSummaryResponse {
	/** Total income */
	total_income: number;
	/** Total expenses */
	total_expenses: number;
	/** Net income (income - expenses) */
	net_income: number;
	/** Number of transactions */
	transaction_count: number;
	/** Average transaction amount */
	average_transaction: number;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * API error structure
 */
export interface ApiError {
	/** Error type */
	type:
		| "Database"
		| "Validation"
		| "Authentication"
		| "Authorization"
		| "NotFound"
		| "Conflict"
		| "InvalidInput"
		| "Security"
		| "Internal"
		| "External";
	/** Error message */
	message: string;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Generic API response wrapper
 */
export type ApiResponse<T> = Promise<T>;

/**
 * Report data structure (flexible for various report types)
 */
export interface ReportData {
	[key: string]:
		| string
		| number
		| boolean
		| null
		| undefined
		| ReportData
		| ReportData[];
}
