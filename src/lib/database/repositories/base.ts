/**
 * Base repository class for common database operations
 * Provides CRUD operations and common query patterns
 */

import {
	executeCommand,
	executeQuery,
	formatDateTimeForDb,
	generateId,
} from "../connection";
import type { BaseEntity, QueryOptions, QueryResult } from "../types";

export abstract class BaseRepository<
	T extends BaseEntity,
	TCreate extends Record<string, unknown>,
	TUpdate extends Record<string, unknown>,
> {
	protected abstract tableName: string;
	protected abstract selectFields: string;

	/**
	 * Define allowed sort fields for this repository
	 * Subclasses must implement this to prevent SQL injection
	 */
	protected abstract getAllowedSortFields(): string[];

	/**
	 * Define allowed fields for create operations
	 * Subclasses must implement this to prevent unexpected columns
	 */
	protected abstract getAllowedCreateFields(): string[];

	/**
	 * Define allowed fields for update operations
	 * Subclasses must implement this to prevent unexpected columns
	 */
	protected abstract getAllowedUpdateFields(): string[];

	/**
	 * Define allowed fields for filter operations
	 * Subclasses must implement this to prevent SQL injection in WHERE clauses
	 */
	protected abstract getAllowedFilterFields(): string[];

	/**
	 * Validate and sanitize sort field to prevent SQL injection
	 * @param field Sort field from user input
	 * @returns Validated field name or default
	 */
	protected validateSortField(field: string): string {
		const allowedFields = this.getAllowedSortFields();
		if (allowedFields.includes(field)) {
			return field;
		}

		// Log potential security issue
		console.warn(`Invalid sort field attempted: ${field}. Using default sort.`);
		return "created_at"; // Safe default
	}

	/**
	 * Validate sort direction to prevent SQL injection
	 * @param direction Sort direction from user input
	 * @returns Validated direction
	 */
	protected validateSortDirection(direction: string): "ASC" | "DESC" {
		const normalizedDirection = direction.toLowerCase();
		return normalizedDirection === "asc" ? "ASC" : "DESC";
	}

	/**
	 * Validate input object keys against allowed fields
	 * @param input Input object to validate
	 * @param allowedFields Array of allowed field names
	 * @param operation Operation type for error messages
	 * @returns Validated input object with only allowed fields
	 */
	protected validateInputFields<TInput extends Record<string, unknown>>(
		input: TInput,
		allowedFields: string[],
		operation: string,
	): Partial<TInput> {
		const inputKeys = Object.keys(input);
		const invalidKeys = inputKeys.filter((key) => !allowedFields.includes(key));

		if (invalidKeys.length > 0) {
			console.warn(
				`Invalid fields attempted in ${operation} operation: ${invalidKeys.join(", ")}. ` +
					`Allowed fields: ${allowedFields.join(", ")}`,
			);
		}

		// Return object with only allowed fields
		const validatedInput: Record<string, unknown> = {};
		for (const key of inputKeys) {
			if (allowedFields.includes(key)) {
				validatedInput[key] = input[key];
			}
		}

		return validatedInput as Partial<TInput>;
	}

	/**
	 * Validate filter fields to prevent SQL injection in WHERE clauses
	 * @param filters Filter conditions to validate
	 * @returns Validated filter object with only allowed fields
	 */
	protected validateFilterFields(
		filters: Record<string, unknown>,
	): Record<string, unknown> {
		const allowedFields = this.getAllowedFilterFields();
		const inputKeys = Object.keys(filters);
		const invalidKeys = inputKeys.filter((key) => !allowedFields.includes(key));

		if (invalidKeys.length > 0) {
			console.warn(
				`Invalid filter fields attempted: ${invalidKeys.join(", ")}. ` +
					`Allowed fields: ${allowedFields.join(", ")}`,
			);
		}

		// Return object with only allowed fields
		const validatedFilters: Record<string, unknown> = {};
		for (const key of inputKeys) {
			if (allowedFields.includes(key)) {
				validatedFilters[key] = filters[key];
			}
		}

		return validatedFilters;
	}

	/**
	 * Build secure ORDER BY clause with field validation
	 * @param sort Sort options from user input
	 * @param tableAlias Optional table alias for prefixing
	 * @returns Secure ORDER BY clause
	 */
	protected buildOrderByClause(
		sort?: { field: string; direction: string },
		tableAlias?: string,
	): string {
		if (!sort) {
			const prefix = tableAlias ? `${tableAlias}.` : "";
			return `ORDER BY ${prefix}created_at DESC`;
		}

		const validatedField = this.validateSortField(sort.field);
		const validatedDirection = this.validateSortDirection(sort.direction);
		const prefix = tableAlias ? `${tableAlias}.` : "";

		return `ORDER BY ${prefix}"${validatedField}" ${validatedDirection}`;
	}

	/**
	 * Find a record by ID
	 * @param id Record ID
	 * @returns Record or null if not found
	 */
	async findById(id: string): Promise<T | null> {
		const query = `SELECT ${this.selectFields} FROM ${this.tableName} WHERE id = $1`;
		const result = await executeQuery<T>(query, [id]);
		return result[0] || null;
	}

	/**
	 * Find all records with optional filtering and pagination
	 * @param filters Filter conditions
	 * @param options Query options (pagination, sorting)
	 * @returns Query result with data and metadata
	 */
	async findAll(
		filters: Record<string, unknown> = {},
		options: QueryOptions = {},
	): Promise<QueryResult<T>> {
		const { page = 1, limit = 50, sort } = options;
		const offset = (page - 1) * limit;

		// Validate filter fields to prevent SQL injection
		const validatedFilters = this.validateFilterFields(filters);

		// Build WHERE clause
		const whereConditions: string[] = [];
		const params: unknown[] = [];
		let paramIndex = 1;

		Object.entries(validatedFilters).forEach(([key, value]) => {
			if (value !== undefined && value !== null) {
				whereConditions.push(`"${key}" = $${paramIndex}`);
				params.push(value);
				paramIndex++;
			}
		});

		const whereClause =
			whereConditions.length > 0
				? `WHERE ${whereConditions.join(" AND ")}`
				: "";

		// Build ORDER BY clause with security validation
		const orderClause = this.buildOrderByClause(sort);

		// Build main query
		const query = `
      SELECT ${this.selectFields} 
      FROM ${this.tableName} 
      ${whereClause} 
      ${orderClause} 
      LIMIT $${paramIndex} OFFSET $${paramIndex + 1}
    `;
		params.push(limit, offset);

		// Build count query
		const countQuery = `
      SELECT COUNT(*) as total 
      FROM ${this.tableName} 
      ${whereClause}
    `;
		const countParams = params.slice(0, -2); // Remove limit and offset

		// Execute both queries
		const [data, countResult] = await Promise.all([
			executeQuery<T>(query, params),
			executeQuery<{ total: number }>(countQuery, countParams),
		]);

		const total = countResult[0]?.total || 0;

		return {
			data,
			total,
			page,
			limit,
		};
	}

	/**
	 * Create a new record
	 * @param input Record data
	 * @returns Created record
	 */
	async create(input: TCreate): Promise<T> {
		// Validate input fields against allowed create fields
		const allowedFields = this.getAllowedCreateFields();
		const validatedInput = this.validateInputFields(
			input,
			allowedFields,
			"create",
		);

		const id = generateId();
		const now = formatDateTimeForDb(new Date());

		const fields = [
			"id",
			"created_at",
			"updated_at",
			...Object.keys(validatedInput),
		];
		const values = [id, now, now, ...Object.values(validatedInput)];
		const placeholders = fields.map((_, index) => `$${index + 1}`).join(", ");

		const query = `
      INSERT INTO ${this.tableName} (${fields.join(", ")})
      VALUES (${placeholders})
    `;

		await executeCommand(query, values);

		// Return the created record
		const created = await this.findById(id);
		if (!created) {
			throw new Error(`Failed to create record in ${this.tableName}`);
		}

		return created;
	}

	/**
	 * Update a record by ID
	 * @param id Record ID
	 * @param input Update data
	 * @returns Updated record
	 */
	async update(id: string, input: Partial<TUpdate>): Promise<T> {
		// Validate input fields against allowed update fields
		const allowedFields = this.getAllowedUpdateFields();
		const validatedInput = this.validateInputFields(
			input,
			allowedFields,
			"update",
		);

		const now = formatDateTimeForDb(new Date());
		const updateData = { ...validatedInput, updated_at: now };

		const fields = Object.keys(updateData);
		const values = Object.values(updateData);
		const setClause = fields
			.map((field, index) => `${field} = $${index + 1}`)
			.join(", ");

		const query = `
      UPDATE ${this.tableName}
      SET ${setClause}
      WHERE id = $${fields.length + 1}
    `;

		const result = await executeCommand(query, [...values, id]);

		if (result.rowsAffected === 0) {
			throw new Error(`Record with id ${id} not found in ${this.tableName}`);
		}

		// Return the updated record
		const updated = await this.findById(id);
		if (!updated) {
			throw new Error(
				`Failed to retrieve updated record from ${this.tableName}`,
			);
		}

		return updated;
	}

	/**
	 * Delete a record by ID
	 * @param id Record ID
	 * @returns True if deleted, false if not found
	 */
	async delete(id: string): Promise<boolean> {
		const query = `DELETE FROM ${this.tableName} WHERE id = $1`;
		const result = await executeCommand(query, [id]);
		return result.rowsAffected > 0;
	}

	/**
	 * Check if a record exists by ID
	 * @param id Record ID
	 * @returns True if exists, false otherwise
	 */
	async exists(id: string): Promise<boolean> {
		const query = `SELECT 1 FROM ${this.tableName} WHERE id = $1 LIMIT 1`;
		const result = await executeQuery(query, [id]);
		return result.length > 0;
	}

	/**
	 * Count records with optional filtering
	 * @param filters Filter conditions
	 * @returns Number of matching records
	 */
	async count(filters: Record<string, unknown> = {}): Promise<number> {
		// Validate filter fields to prevent SQL injection
		const validatedFilters = this.validateFilterFields(filters);

		const whereConditions: string[] = [];
		const params: unknown[] = [];
		let paramIndex = 1;

		Object.entries(validatedFilters).forEach(([key, value]) => {
			if (value !== undefined && value !== null) {
				whereConditions.push(`"${key}" = $${paramIndex}`);
				params.push(value);
				paramIndex++;
			}
		});

		const whereClause =
			whereConditions.length > 0
				? `WHERE ${whereConditions.join(" AND ")}`
				: "";

		const query = `SELECT COUNT(*) as total FROM ${this.tableName} ${whereClause}`;
		const result = await executeQuery<{ total: number }>(query, params);
		return result[0]?.total || 0;
	}

	/**
	 * Find records by a specific field value
	 * @param field Field name
	 * @param value Field value
	 * @param options Query options
	 * @returns Matching records
	 */
	async findBy(
		field: string,
		value: unknown,
		options: QueryOptions = {},
	): Promise<T[]> {
		// Validate field name to prevent SQL injection
		const allowedFields = this.getAllowedFilterFields();
		if (!allowedFields.includes(field)) {
			console.warn(
				`Invalid field attempted in findBy: ${field}. ` +
					`Allowed fields: ${allowedFields.join(", ")}`,
			);
			return [];
		}

		const { limit = 50, sort } = options;

		const orderClause = this.buildOrderByClause(sort);

		const query = `
      SELECT ${this.selectFields}
      FROM ${this.tableName}
      WHERE "${field}" = $1
      ${orderClause}
      LIMIT $2
    `;

		return executeQuery<T>(query, [value, limit]);
	}

	/**
	 * Find first record matching criteria
	 * @param filters Filter conditions
	 * @returns First matching record or null
	 */
	async findFirst(filters: Record<string, unknown> = {}): Promise<T | null> {
		const result = await this.findAll(filters, { limit: 1 });
		return result.data[0] || null;
	}
}
