/**
 * Database types for the Fiscus personal finance application
 * These types correspond to the database schema defined in the SQL migrations
 */

// Base interface for all database entities
export interface BaseEntity {
	id: string;
	created_at: string;
	updated_at?: string;
}

// User entity
export interface User extends BaseEntity {
	username: string;
	email?: string;
	password_hash: string;
}

// Account type entity
export interface AccountType extends Omit<BaseEntity, "updated_at"> {
	name: string;
	description?: string;
	is_asset: boolean;
}

// Account entity
export interface Account extends BaseEntity {
	user_id: string;
	account_type_id: string;
	name: string;
	description?: string;
	initial_balance: number;
	current_balance: number;
	currency: string;
	is_active: boolean;
	institution_name?: string;
	account_number?: string;
}

// Category entity
export interface Category extends BaseEntity {
	user_id: string;
	name: string;
	description?: string;
	color?: string;
	icon?: string;
	parent_category_id?: string;
	is_income: boolean;
	is_active: boolean;
}

// Transaction entity
export interface Transaction extends BaseEntity {
	user_id: string;
	account_id: string;
	category_id?: string;
	amount: number;
	description: string;
	notes?: string;
	transaction_date: string; // ISO date string
	transaction_type: "income" | "expense" | "transfer";
	status: "pending" | "completed" | "cancelled";
	reference_number?: string;
	payee?: string;
	tags?: string; // JSON string of tags array
}

// Transfer entity
export interface Transfer extends Omit<BaseEntity, "updated_at"> {
	user_id: string;
	from_account_id: string;
	to_account_id: string;
	from_transaction_id: string;
	to_transaction_id: string;
	amount: number;
	description?: string;
	transfer_date: string; // ISO date string
}

// Budget period entity
export interface BudgetPeriod extends BaseEntity {
	user_id: string;
	name: string;
	start_date: string; // ISO date string
	end_date: string; // ISO date string
	is_active: boolean;
}

// Budget entity
export interface Budget extends BaseEntity {
	user_id: string;
	budget_period_id: string;
	category_id: string;
	allocated_amount: number;
	spent_amount: number;
	notes?: string;
}

// Goal entity
export interface Goal extends BaseEntity {
	user_id: string;
	name: string;
	description?: string;
	target_amount: number;
	current_amount: number;
	target_date?: string; // ISO date string
	priority: number; // 1-5
	status: "active" | "completed" | "paused" | "cancelled";
	category?: string;
}

// Input types for creating new entities (without generated fields)
export type CreateUserInput = Omit<User, keyof BaseEntity>;
export type CreateAccountInput = Omit<Account, keyof BaseEntity>;
export type CreateCategoryInput = Omit<Category, keyof BaseEntity>;
export type CreateTransactionInput = Omit<Transaction, keyof BaseEntity>;
export type CreateTransferInput = Omit<
	Transfer,
	keyof BaseEntity | "created_at"
>;
export type CreateBudgetPeriodInput = Omit<BudgetPeriod, keyof BaseEntity>;
export type CreateBudgetInput = Omit<Budget, keyof BaseEntity>;
export type CreateGoalInput = Omit<Goal, keyof BaseEntity>;

// Update types (partial updates with required id)
export type UpdateAccountInput = Partial<CreateAccountInput> & { id: string };
export type UpdateCategoryInput = Partial<CreateCategoryInput> & { id: string };
export type UpdateTransactionInput = Partial<CreateTransactionInput> & {
	id: string;
};
export type UpdateBudgetPeriodInput = Partial<CreateBudgetPeriodInput> & {
	id: string;
};
export type UpdateBudgetInput = Partial<CreateBudgetInput> & { id: string };
export type UpdateGoalInput = Partial<CreateGoalInput> & { id: string };

// Extended types with joined data for UI
export interface AccountWithType extends Account {
	account_type: AccountType;
}

export interface TransactionWithDetails extends Transaction {
	account: Account;
	category?: Category;
}

export interface BudgetWithDetails extends Budget {
	category: Category;
	budget_period: BudgetPeriod;
}

// Query filter types
export interface TransactionFilters {
	account_id?: string;
	category_id?: string;
	transaction_type?: Transaction["transaction_type"];
	status?: Transaction["status"];
	start_date?: string;
	end_date?: string;
	min_amount?: number;
	max_amount?: number;
	search?: string; // Search in description, notes, payee
}

export interface AccountFilters {
	account_type_id?: string;
	is_active?: boolean;
	currency?: string;
}

export interface CategoryFilters {
	is_income?: boolean;
	is_active?: boolean;
	parent_category_id?: string;
}

// Pagination and sorting
export interface PaginationOptions {
	page?: number;
	limit?: number;
}

export interface SortOptions {
	field: string;
	direction: "asc" | "desc";
}

export interface QueryOptions extends PaginationOptions {
	sort?: SortOptions;
}

// Database operation result types
export interface QueryResult<T = unknown> {
	data: T[];
	total?: number;
	page?: number;
	limit?: number;
}

export interface DatabaseError {
	message: string;
	code?: string;
	details?: unknown;
}

// Utility types for financial calculations
export interface AccountBalance {
	account_id: string;
	account_name: string;
	current_balance: number;
	currency: string;
}

export interface CategorySpending {
	category_id: string;
	category_name: string;
	total_spent: number;
	transaction_count: number;
}

export interface MonthlySpending {
	month: string; // YYYY-MM format
	total_income: number;
	total_expenses: number;
	net_income: number;
}

// Dashboard summary types
export interface DashboardSummary {
	total_assets: number;
	total_liabilities: number;
	net_worth: number;
	monthly_income: number;
	monthly_expenses: number;
	recent_transactions: TransactionWithDetails[];
	account_balances: AccountBalance[];
	top_categories: CategorySpending[];
}
