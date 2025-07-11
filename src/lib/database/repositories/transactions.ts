/**
 * Transaction repository for managing financial transactions
 */

import { executeQuery, executeTransaction } from "../connection";
import type {
	CategorySpending,
	CreateTransactionInput,
	MonthlySpending,
	QueryOptions,
	QueryResult,
	Transaction,
	TransactionFilters,
	TransactionWithDetails,
	UpdateTransactionInput,
} from "../types";

// Internal types for database row data
interface TransactionRowData {
	id: string;
	user_id: string;
	account_id: string;
	category_id: string | null;
	amount: number;
	description: string;
	notes: string | null;
	transaction_date: string;
	transaction_type: string;
	status: string;
	payee: string | null;
	reference_number: string | null;
	tags: string | null;
	created_at: string;
	updated_at: string;
	account_name: string;
	account_type_id: string;
	account_user_id: string;
	account_initial_balance: number;
	account_current_balance: number;
	account_currency: string;
	account_is_active: boolean;
	account_institution_name: string | null;
	account_account_number: string | null;
	account_description: string | null;
	account_created_at: string;
	account_updated_at: string | null;
	category_name: string | null;
	category_color: string | null;
	category_icon: string | null;
	category_user_id: string | null;
	category_description: string | null;
	category_parent_category_id: string | null;
	category_is_income: boolean | null;
	category_is_active: boolean | null;
	category_created_at: string | null;
	category_updated_at: string | null;
}

import { BaseRepository } from "./base";

export class TransactionRepository extends BaseRepository<
	Transaction,
	CreateTransactionInput,
	UpdateTransactionInput
> {
	protected tableName = "transactions";
	protected selectFields = `
    id, user_id, account_id, category_id, amount, description, notes,
    transaction_date, transaction_type, status, reference_number, payee, tags,
    created_at, updated_at
  `;

	/**
	 * Define allowed sort fields to prevent SQL injection
	 */
	protected getAllowedSortFields(): string[] {
		return [
			"id",
			"amount",
			"description",
			"notes",
			"transaction_date",
			"transaction_type",
			"status",
			"reference_number",
			"payee",
			"created_at",
			"updated_at",
		];
	}

	/**
	 * Define allowed fields for create operations
	 */
	protected getAllowedCreateFields(): string[] {
		return [
			"user_id",
			"account_id",
			"category_id",
			"amount",
			"description",
			"notes",
			"transaction_date",
			"transaction_type",
			"status",
			"reference_number",
			"payee",
			"tags",
		];
	}

	/**
	 * Define allowed fields for update operations
	 */
	protected getAllowedUpdateFields(): string[] {
		return [
			"account_id",
			"category_id",
			"amount",
			"description",
			"notes",
			"transaction_date",
			"transaction_type",
			"status",
			"reference_number",
			"payee",
			"tags",
		];
	}

	/**
	 * Define allowed fields for filter operations
	 */
	protected getAllowedFilterFields(): string[] {
		return [
			"id",
			"user_id",
			"account_id",
			"category_id",
			"amount",
			"description",
			"notes",
			"transaction_date",
			"transaction_type",
			"status",
			"reference_number",
			"payee",
			"tags",
			"created_at",
			"updated_at",
		];
	}

	/**
	 * Find transactions with account and category details
	 * @param userId User ID
	 * @param filters Transaction filters
	 * @param options Query options
	 * @returns Transactions with details
	 */
	async findWithDetails(
		userId: string,
		filters: TransactionFilters = {},
		options: QueryOptions = {},
	): Promise<QueryResult<TransactionWithDetails>> {
		const { page = 1, limit = 50, sort } = options;
		const offset = (page - 1) * limit;

		// Build WHERE clause
		const whereConditions = ["t.user_id = $1"];
		const params: (string | number | boolean)[] = [userId];
		let paramIndex = 2;

		if (filters.account_id) {
			whereConditions.push(`t.account_id = $${paramIndex}`);
			params.push(filters.account_id);
			paramIndex++;
		}

		if (filters.category_id) {
			whereConditions.push(`t.category_id = $${paramIndex}`);
			params.push(filters.category_id);
			paramIndex++;
		}

		if (filters.transaction_type) {
			whereConditions.push(`t.transaction_type = $${paramIndex}`);
			params.push(filters.transaction_type);
			paramIndex++;
		}

		if (filters.status) {
			whereConditions.push(`t.status = $${paramIndex}`);
			params.push(filters.status);
			paramIndex++;
		}

		if (filters.start_date) {
			whereConditions.push(`t.transaction_date >= $${paramIndex}`);
			params.push(filters.start_date);
			paramIndex++;
		}

		if (filters.end_date) {
			whereConditions.push(`t.transaction_date <= $${paramIndex}`);
			params.push(filters.end_date);
			paramIndex++;
		}

		if (filters.min_amount !== undefined) {
			whereConditions.push(`ABS(t.amount) >= $${paramIndex}`);
			params.push(filters.min_amount);
			paramIndex++;
		}

		if (filters.max_amount !== undefined) {
			whereConditions.push(`ABS(t.amount) <= $${paramIndex}`);
			params.push(filters.max_amount);
			paramIndex++;
		}

		if (filters.search) {
			whereConditions.push(`(
        t.description LIKE $${paramIndex} OR 
        t.notes LIKE $${paramIndex} OR 
        t.payee LIKE $${paramIndex}
      )`);
			params.push(`%${filters.search}%`);
			paramIndex++;
		}

		const whereClause = `WHERE ${whereConditions.join(" AND ")}`;

		// Build ORDER BY clause with security validation
		const orderClause = sort
			? this.buildOrderByClause(sort, "t")
			: "ORDER BY t.transaction_date DESC, t.created_at DESC";

		// Main query with JOINs
		const query = `
      SELECT 
        t.id, t.user_id, t.account_id, t.category_id, t.amount, t.description, t.notes,
        t.transaction_date, t.transaction_type, t.status, t.reference_number, t.payee, t.tags,
        t.created_at, t.updated_at,
		a.name as account_name, a.account_type_id, a.user_id as account_user_id,
        a.initial_balance as account_initial_balance, a.current_balance as account_current_balance,
        a.currency as account_currency, a.is_active as account_is_active,
        a.institution_name as account_institution_name, a.account_number as account_account_number,
        a.description as account_description, a.created_at as account_created_at,
        a.updated_at as account_updated_at,
        c.id as category_id, c.name as category_name, c.color as category_color, 
        c.icon as category_icon, c.user_id as category_user_id,
        c.description as category_description, c.parent_category_id as category_parent_category_id,
        c.is_income as category_is_income, c.is_active as category_is_active,
        c.created_at as category_created_at, c.updated_at as category_updated_at
      FROM transactions t
      JOIN accounts a ON t.account_id = a.id
      LEFT JOIN categories c ON t.category_id = c.id
      ${whereClause}
      ${orderClause}
      LIMIT $${paramIndex} OFFSET $${paramIndex + 1}
    `;
		params.push(limit, offset);

		// Count query
		const countQuery = `
      SELECT COUNT(*) as total
      FROM transactions t
      ${whereClause}
    `;
		const countParams = params.slice(0, -2);

		const [rawData, countResult] = await Promise.all([
			executeQuery<TransactionRowData>(query, params),
			executeQuery<{ total: number }>(countQuery, countParams),
		]);

		// Transform the joined data
		const data: TransactionWithDetails[] = rawData.map(
			(row: TransactionRowData) => ({
				id: row.id,
				user_id: row.user_id,
				account_id: row.account_id,
				category_id: row.category_id ?? undefined,
				amount: row.amount,
				description: row.description,
				notes: row.notes ?? undefined,
				transaction_date: row.transaction_date,
				transaction_type: row.transaction_type as
					| "income"
					| "expense"
					| "transfer",
				status: row.status as "pending" | "completed" | "cancelled",
				reference_number: row.reference_number ?? undefined,
				payee: row.payee ?? undefined,
				tags: row.tags ?? undefined,
				created_at: row.created_at,
				updated_at: row.updated_at ?? undefined,
				account: {
					id: row.account_id,
					user_id: row.account_user_id,
					account_type_id: row.account_type_id,
					name: row.account_name,
					description: row.account_description ?? undefined,
					initial_balance: row.account_initial_balance,
					current_balance: row.account_current_balance,
					currency: row.account_currency,
					is_active: row.account_is_active,
					institution_name: row.account_institution_name ?? undefined,
					account_number: row.account_account_number ?? undefined,
					created_at: row.account_created_at,
					updated_at: row.account_updated_at ?? undefined,
				},
				category:
					row.category_id && row.category_user_id
						? {
								id: row.category_id,
								user_id: row.category_user_id,
								name: row.category_name ?? "",
								description: row.category_description ?? undefined,
								color: row.category_color ?? undefined,
								icon: row.category_icon ?? undefined,
								parent_category_id:
									row.category_parent_category_id ?? undefined,
								is_income: row.category_is_income ?? false,
								is_active: row.category_is_active ?? true,
								created_at: row.category_created_at ?? "",
								updated_at: row.category_updated_at ?? undefined,
							}
						: undefined,
			}),
		);

		const total = countResult[0]?.total || 0;

		return {
			data,
			total,
			page,
			limit,
		};
	}

	/**
	 * Create a transaction and update account balance
	 * @param input Transaction data
	 * @returns Created transaction
	 */
	async createWithBalanceUpdate(
		input: CreateTransactionInput,
	): Promise<Transaction> {
		const commands = [
			{
				query: `
          INSERT INTO transactions (
            id, user_id, account_id, category_id, amount, description, notes,
            transaction_date, transaction_type, status, reference_number, payee, tags,
            created_at, updated_at
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        `,
				params: [
					crypto.randomUUID(),
					input.user_id,
					input.account_id,
					input.category_id,
					input.amount,
					input.description,
					input.notes,
					input.transaction_date,
					input.transaction_type,
					input.status || "completed",
					input.reference_number,
					input.payee,
					input.tags,
					new Date().toISOString(),
					new Date().toISOString(),
				],
			},
			{
				query: `
          UPDATE accounts 
          SET current_balance = current_balance + $1, updated_at = $2 
          WHERE id = $3
        `,
				params: [input.amount, new Date().toISOString(), input.account_id],
			},
		];

		await executeTransaction(commands);
		const transactionId = commands[0].params?.[0] as string;

		const created = await this.findById(transactionId);
		if (!created) {
			throw new Error("Failed to create transaction");
		}

		return created;
	}

	/**
	 * Get recent transactions for a user
	 * @param userId User ID
	 * @param limit Number of transactions to return
	 * @returns Recent transactions
	 */
	async getRecent(
		userId: string,
		limit: number = 10,
	): Promise<TransactionWithDetails[]> {
		const result = await this.findWithDetails(userId, {}, { limit });
		return result.data;
	}

	/**
	 * Get category spending summary
	 * @param userId User ID
	 * @param startDate Start date (optional)
	 * @param endDate End date (optional)
	 * @returns Category spending data
	 */
	async getCategorySpending(
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<CategorySpending[]> {
		const whereConditions = ["t.user_id = $1", "t.transaction_type = $2"];
		const params: (string | number | boolean)[] = [userId, "expense"];
		let paramIndex = 3;

		if (startDate) {
			whereConditions.push(`t.transaction_date >= $${paramIndex}`);
			params.push(startDate);
			paramIndex++;
		}

		if (endDate) {
			whereConditions.push(`t.transaction_date <= $${paramIndex}`);
			params.push(endDate);
			paramIndex++;
		}

		const whereClause = `WHERE ${whereConditions.join(" AND ")}`;

		const query = `
      SELECT 
        c.id as category_id,
        c.name as category_name,
        SUM(ABS(t.amount)) as total_spent,
        COUNT(t.id) as transaction_count
      FROM transactions t
      JOIN categories c ON t.category_id = c.id
      ${whereClause}
      GROUP BY c.id, c.name
      ORDER BY total_spent DESC
    `;

		return executeQuery<CategorySpending>(query, params);
	}

	/**
	 * Get monthly spending summary
	 * @param userId User ID
	 * @param year Year (optional, defaults to current year)
	 * @returns Monthly spending data
	 */
	async getMonthlySpending(
		userId: string,
		year?: number,
	): Promise<MonthlySpending[]> {
		const currentYear = year || new Date().getFullYear();

		const query = `
      SELECT 
        strftime('%Y-%m', t.transaction_date) as month,
        COALESCE(SUM(CASE WHEN t.transaction_type = 'income' THEN t.amount ELSE 0 END), 0) as total_income,
        COALESCE(SUM(CASE WHEN t.transaction_type = 'expense' THEN ABS(t.amount) ELSE 0 END), 0) as total_expenses,
        COALESCE(SUM(CASE WHEN t.transaction_type = 'income' THEN t.amount ELSE -ABS(t.amount) END), 0) as net_income
      FROM transactions t
      WHERE t.user_id = $1 
        AND strftime('%Y', t.transaction_date) = $2
        AND t.status = 'completed'
      GROUP BY strftime('%Y-%m', t.transaction_date)
      ORDER BY month
    `;

		return executeQuery<MonthlySpending>(query, [
			userId,
			currentYear.toString(),
		]);
	}

	/**
	 * Find transactions by account
	 * @param accountId Account ID
	 * @param options Query options
	 * @returns Account transactions
	 */
	async findByAccount(
		accountId: string,
		options: QueryOptions = {},
	): Promise<Transaction[]> {
		return this.findBy("account_id", accountId, options);
	}

	/**
	 * Find transactions by category
	 * @param categoryId Category ID
	 * @param options Query options
	 * @returns Category transactions
	 */
	async findByCategory(
		categoryId: string,
		options: QueryOptions = {},
	): Promise<Transaction[]> {
		return this.findBy("category_id", categoryId, options);
	}

	/**
	 * Get total income for a period
	 * @param userId User ID
	 * @param startDate Start date
	 * @param endDate End date
	 * @returns Total income
	 */
	async getTotalIncome(
		userId: string,
		startDate: string,
		endDate: string,
	): Promise<number> {
		const query = `
      SELECT COALESCE(SUM(amount), 0) as total
      FROM transactions
      WHERE user_id = $1 
        AND transaction_type = 'income'
        AND transaction_date BETWEEN $2 AND $3
        AND status = 'completed'
    `;

		const result = await executeQuery<{ total: number }>(query, [
			userId,
			startDate,
			endDate,
		]);
		return result[0]?.total || 0;
	}

	/**
	 * Get total expenses for a period
	 * @param userId User ID
	 * @param startDate Start date
	 * @param endDate End date
	 * @returns Total expenses
	 */
	async getTotalExpenses(
		userId: string,
		startDate: string,
		endDate: string,
	): Promise<number> {
		const query = `
      SELECT COALESCE(SUM(ABS(amount)), 0) as total
      FROM transactions
      WHERE user_id = $1 
        AND transaction_type = 'expense'
        AND transaction_date BETWEEN $2 AND $3
        AND status = 'completed'
    `;

		const result = await executeQuery<{ total: number }>(query, [
			userId,
			startDate,
			endDate,
		]);
		return result[0]?.total || 0;
	}
}
