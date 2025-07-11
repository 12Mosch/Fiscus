/**
 * Account repository for managing financial accounts
 */

import { executeCommand, executeQuery } from "../connection";
import type {
	Account,
	AccountBalance,
	AccountFilters,
	AccountWithType,
	CreateAccountInput,
	QueryOptions,
	QueryResult,
	UpdateAccountInput,
} from "../types";
import { BaseRepository } from "./base";

export class AccountRepository extends BaseRepository<
	Account,
	CreateAccountInput,
	UpdateAccountInput
> {
	protected tableName = "accounts";
	protected selectFields = `
    id, user_id, account_type_id, name, description,
    initial_balance, current_balance, currency, is_active,
    institution_name, account_number, created_at, updated_at
  `;

	/**
	 * Define allowed sort fields to prevent SQL injection
	 */
	protected getAllowedSortFields(): string[] {
		return [
			"id",
			"name",
			"description",
			"initial_balance",
			"current_balance",
			"currency",
			"is_active",
			"institution_name",
			"account_number",
			"created_at",
			"updated_at",
		];
	}

	/**
	 * Find accounts with their account type information
	 * @param userId User ID
	 * @param filters Optional filters
	 * @param options Query options
	 * @returns Accounts with type information
	 */
	async findWithType(
		userId: string,
		filters: AccountFilters = {},
		options: QueryOptions = {},
	): Promise<QueryResult<AccountWithType>> {
		const { page = 1, limit = 50, sort } = options;
		const offset = (page - 1) * limit;

		// Build WHERE clause
		const whereConditions = ["a.user_id = $1"];
		const params: unknown[] = [userId];
		let paramIndex = 2;

		if (filters.account_type_id) {
			whereConditions.push(`a.account_type_id = $${paramIndex}`);
			params.push(filters.account_type_id);
			paramIndex++;
		}

		if (filters.is_active !== undefined) {
			whereConditions.push(`a.is_active = $${paramIndex}`);
			params.push(filters.is_active);
			paramIndex++;
		}

		if (filters.currency) {
			whereConditions.push(`a.currency = $${paramIndex}`);
			params.push(filters.currency);
			paramIndex++;
		}

		const whereClause = `WHERE ${whereConditions.join(" AND ")}`;

		// Build ORDER BY clause with security validation
		const orderClause = this.buildOrderByClause(sort, "a");

		// Main query with JOIN
		const query = `
      SELECT 
        a.id, a.user_id, a.account_type_id, a.name, a.description,
        a.initial_balance, a.current_balance, a.currency, a.is_active,
        a.institution_name, a.account_number, a.created_at, a.updated_at,
        at.id as account_type_id, at.name as account_type_name,
		at.description as account_type_description, at.is_asset as account_type_is_asset,
		at.created_at as account_type_created_at      FROM accounts a
      JOIN account_types at ON a.account_type_id = at.id
      ${whereClause}
      ${orderClause}
      LIMIT $${paramIndex} OFFSET $${paramIndex + 1}
    `;
		params.push(limit, offset);

		// Count query
		const countQuery = `
      SELECT COUNT(*) as total
      FROM accounts a
      ${whereClause}
    `;
		const countParams = params.slice(0, -2);

		const [rawData, countResult] = await Promise.all([
			executeQuery<Record<string, unknown>>(query, params),
			executeQuery<{ total: number }>(countQuery, countParams),
		]);

		// Transform the joined data
		const data: AccountWithType[] = rawData.map(
			(row: Record<string, unknown>) => ({
				id: row.id as string,
				user_id: row.user_id as string,
				account_type_id: row.account_type_id as string,
				name: row.name as string,
				description: row.description as string | undefined,
				initial_balance: row.initial_balance as number,
				current_balance: row.current_balance as number,
				currency: row.currency as string,
				is_active: row.is_active as boolean,
				institution_name: row.institution_name as string | undefined,
				account_number: row.account_number as string | undefined,
				created_at: row.created_at as string,
				updated_at: row.updated_at as string | undefined,
				account_type: {
					id: row.account_type_id as string,
					name: row.account_type_name as string,
					description: row.account_type_description as string | undefined,
					is_asset: row.account_type_is_asset as boolean,
					created_at: row.created_type_at as string,
				},
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
	 * Find accounts by user ID
	 * @param userId User ID
	 * @param options Query options
	 * @returns User's accounts
	 */
	async findByUserId(
		userId: string,
		options: QueryOptions = {},
	): Promise<Account[]> {
		return this.findBy("user_id", userId, options);
	}

	/**
	 * Get account balances summary for a user
	 * @param userId User ID
	 * @returns Array of account balances
	 */
	async getAccountBalances(userId: string): Promise<AccountBalance[]> {
		const query = `
      SELECT 
        a.id as account_id,
        a.name as account_name,
        a.current_balance,
        a.currency
      FROM accounts a
      WHERE a.user_id = $1 AND a.is_active = true
      ORDER BY a.name
    `;

		return executeQuery<AccountBalance>(query, [userId]);
	}

	/**
	 * Update account balance
	 * @param accountId Account ID
	 * @param newBalance New balance amount
	 * @returns Updated account
	 */
	async updateBalance(accountId: string, newBalance: number): Promise<Account> {
		const query = `
      UPDATE accounts 
      SET current_balance = $1, updated_at = $2 
      WHERE id = $3
    `;

		const now = new Date().toISOString();
		const result = await executeCommand(query, [newBalance, now, accountId]);

		if (result.rowsAffected === 0) {
			throw new Error(`Account with id ${accountId} not found`);
		}

		const updated = await this.findById(accountId);
		if (!updated) {
			throw new Error("Failed to retrieve updated account");
		}

		return updated;
	}

	/**
	 * Get total assets for a user
	 * @param userId User ID
	 * @returns Total asset value
	 */
	async getTotalAssets(userId: string): Promise<number> {
		const query = `
      SELECT COALESCE(SUM(a.current_balance), 0) as total
      FROM accounts a
      JOIN account_types at ON a.account_type_id = at.id
      WHERE a.user_id = $1 AND a.is_active = true AND at.is_asset = true
    `;

		const result = await executeQuery<{ total: number }>(query, [userId]);
		return result[0]?.total || 0;
	}

	/**
	 * Get total liabilities for a user
	 * @param userId User ID
	 * @returns Total liability value
	 */
	async getTotalLiabilities(userId: string): Promise<number> {
		const query = `
      SELECT COALESCE(SUM(ABS(a.current_balance)), 0) as total
      FROM accounts a
      JOIN account_types at ON a.account_type_id = at.id
      WHERE a.user_id = $1 AND a.is_active = true AND at.is_asset = false
    `;

		const result = await executeQuery<{ total: number }>(query, [userId]);
		return result[0]?.total || 0;
	}

	/**
	 * Calculate net worth for a user
	 * @param userId User ID
	 * @returns Net worth (assets - liabilities)
	 */
	async getNetWorth(userId: string): Promise<number> {
		const [assets, liabilities] = await Promise.all([
			this.getTotalAssets(userId),
			this.getTotalLiabilities(userId),
		]);

		return assets - liabilities;
	}

	/**
	 * Find accounts by type
	 * @param userId User ID
	 * @param accountTypeId Account type ID
	 * @param options Query options
	 * @returns Accounts of specified type
	 */
	async findByType(
		userId: string,
		accountTypeId: string,
		options: QueryOptions = {},
	): Promise<Account[]> {
		const { limit = 50, sort } = options;

		const orderClause = this.buildOrderByClause(sort);

		const query = `
      SELECT ${this.selectFields}
      FROM ${this.tableName}
      WHERE user_id = $1 AND account_type_id = $2
      ${orderClause}
      LIMIT $3
    `;

		return executeQuery<Account>(query, [userId, accountTypeId, limit]);
	}

	/**
	 * Toggle account active status
	 * @param accountId Account ID
	 * @returns Updated account
	 */
	async toggleActive(accountId: string): Promise<Account> {
		const account = await this.findById(accountId);
		if (!account) {
			throw new Error(`Account with id ${accountId} not found`);
		}

		return this.update(accountId, { is_active: !account.is_active });
	}
}
