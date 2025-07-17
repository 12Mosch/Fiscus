/**
 * Client-side validation utilities for Fiscus application
 * Provides comprehensive input validation that mirrors server-side validation
 */

import type {
	CreateAccountRequest,
	CreateBudgetRequest,
	CreateCategoryRequest,
	CreateGoalRequest,
	CreateTransactionRequest,
	CreateTransferRequest,
	CreateUserRequest,
	TransactionType,
} from "../types/api";

/**
 * Validation error interface
 */
export interface ValidationError {
	field: string;
	message: string;
	code: string;
}

/**
 * Validation result interface
 */
export interface ValidationResult {
	isValid: boolean;
	errors: ValidationError[];
}

/**
 * Base validation utilities
 */
export namespace Validator {
	/**
	 * Validate string length and content
	 */
	export function validateString(
		value: string,
		fieldName: string,
		minLength: number,
		maxLength: number,
		required: boolean = true,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (required && (!value || value.trim().length === 0)) {
			errors.push({
				field: fieldName,
				message: `${fieldName} is required`,
				code: "REQUIRED",
			});
			return errors;
		}

		if (!required && (!value || value.trim().length === 0)) {
			return errors; // Optional field, no validation needed
		}

		const trimmedValue = value.trim();

		if (trimmedValue.length < minLength) {
			errors.push({
				field: fieldName,
				message: `${fieldName} must be at least ${minLength} characters`,
				code: "MIN_LENGTH",
			});
		}

		if (trimmedValue.length > maxLength) {
			errors.push({
				field: fieldName,
				message: `${fieldName} cannot exceed ${maxLength} characters`,
				code: "MAX_LENGTH",
			});
		}

		return errors;
	}

	/**
	 * Validate email format
	 */
	export function validateEmail(
		email: string,
		required: boolean = false,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!email || email.trim().length === 0) {
			if (required) {
				errors.push({
					field: "email",
					message: "Email is required",
					code: "REQUIRED",
				});
			}
			return errors;
		}

		// More strict email validation
		const emailRegex =
			/^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/;
		const trimmedEmail = email.trim();

		// Check for consecutive dots
		if (trimmedEmail.includes("..")) {
			errors.push({
				field: "email",
				message: "Invalid email format",
				code: "INVALID_FORMAT",
			});
		} else if (!emailRegex.test(trimmedEmail)) {
			errors.push({
				field: "email",
				message: "Invalid email format",
				code: "INVALID_FORMAT",
			});
		}

		return errors;
	}

	/**
	 * Validate UUID format
	 */
	export function validateUUID(
		value: string,
		fieldName: string,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!value || value.trim().length === 0) {
			errors.push({
				field: fieldName,
				message: `${fieldName} is required`,
				code: "REQUIRED",
			});
			return errors;
		}

		const uuidRegex =
			/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
		if (!uuidRegex.test(value.trim())) {
			errors.push({
				field: fieldName,
				message: `Invalid ${fieldName} format`,
				code: "INVALID_FORMAT",
			});
		}

		return errors;
	}

	/**
	 * Validate amount (must be positive for most operations)
	 */
	export function validateAmount(
		amount: number,
		fieldName: string,
		allowNegative: boolean = false,
		allowZero: boolean = true,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (typeof amount !== "number" || Number.isNaN(amount)) {
			errors.push({
				field: fieldName,
				message: `${fieldName} must be a valid number`,
				code: "INVALID_TYPE",
			});
			return errors;
		}

		if (!allowNegative && amount < 0) {
			errors.push({
				field: fieldName,
				message: `${fieldName} cannot be negative`,
				code: "NEGATIVE_VALUE",
			});
		}

		if (!allowZero && amount === 0) {
			errors.push({
				field: fieldName,
				message: `${fieldName} must be greater than zero`,
				code: "ZERO_VALUE",
			});
		}

		// Check for reasonable limits (prevent overflow)
		const maxAmount = 999_999_999_999;
		if (Math.abs(amount) > maxAmount) {
			errors.push({
				field: fieldName,
				message: `${fieldName} exceeds maximum allowed value`,
				code: "MAX_VALUE",
			});
		}

		return errors;
	}

	/**
	 * Validate date string (YYYY-MM-DD format)
	 */
	export function validateDate(
		dateStr: string,
		fieldName: string,
		required: boolean = true,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!dateStr || dateStr.trim().length === 0) {
			if (required) {
				errors.push({
					field: fieldName,
					message: `${fieldName} is required`,
					code: "REQUIRED",
				});
			}
			return errors;
		}

		const dateRegex = /^\d{4}-\d{2}-\d{2}$/;
		if (!dateRegex.test(dateStr)) {
			errors.push({
				field: fieldName,
				message: `${fieldName} must be in YYYY-MM-DD format`,
				code: "INVALID_FORMAT",
			});
			return errors;
		}

		// Parse date components to validate they're actually valid
		const [year, month, day] = dateStr.split("-").map(Number);
		const date = new Date(year, month - 1, day); // month is 0-indexed in JS Date

		// Check if the date components match what we parsed (catches invalid dates like Feb 30)
		if (
			Number.isNaN(date.getTime()) ||
			date.getFullYear() !== year ||
			date.getMonth() !== month - 1 ||
			date.getDate() !== day
		) {
			errors.push({
				field: fieldName,
				message: `${fieldName} is not a valid date`,
				code: "INVALID_DATE",
			});
		}

		return errors;
	}

	/**
	 * Validate datetime string (ISO 8601 format)
	 */
	export function validateDateTime(
		dateTimeStr: string,
		fieldName: string,
	): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!dateTimeStr || dateTimeStr.trim().length === 0) {
			errors.push({
				field: fieldName,
				message: `${fieldName} is required`,
				code: "REQUIRED",
			});
			return errors;
		}

		const date = new Date(dateTimeStr);
		if (Number.isNaN(date.getTime())) {
			errors.push({
				field: fieldName,
				message: `${fieldName} must be a valid ISO 8601 datetime`,
				code: "INVALID_FORMAT",
			});
		}

		return errors;
	}

	/**
	 * Validate currency code (3 characters)
	 */
	export function validateCurrency(currency: string): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!currency || currency.trim().length === 0) {
			errors.push({
				field: "currency",
				message: "Currency is required",
				code: "REQUIRED",
			});
			return errors;
		}

		const currencyRegex = /^[A-Z]{3}$/;
		if (!currencyRegex.test(currency.trim())) {
			errors.push({
				field: "currency",
				message: "Currency must be a 3-letter ISO code (e.g., USD)",
				code: "INVALID_FORMAT",
			});
		}

		return errors;
	}

	/**
	 * Validate password strength
	 */
	export function validatePassword(password: string): ValidationError[] {
		const errors: ValidationError[] = [];

		if (!password || password.length === 0) {
			errors.push({
				field: "password",
				message: "Password is required",
				code: "REQUIRED",
			});
			return errors;
		}

		if (password.length < 8) {
			errors.push({
				field: "password",
				message: "Password must be at least 8 characters",
				code: "MIN_LENGTH",
			});
		}

		if (password.length > 128) {
			errors.push({
				field: "password",
				message: "Password cannot exceed 128 characters",
				code: "MAX_LENGTH",
			});
		}

		// Check for at least one uppercase letter
		if (!/[A-Z]/.test(password)) {
			errors.push({
				field: "password",
				message: "Password must contain at least one uppercase letter",
				code: "MISSING_UPPERCASE",
			});
		}

		// Check for at least one lowercase letter
		if (!/[a-z]/.test(password)) {
			errors.push({
				field: "password",
				message: "Password must contain at least one lowercase letter",
				code: "MISSING_LOWERCASE",
			});
		}

		// Check for at least one number
		if (!/\d/.test(password)) {
			errors.push({
				field: "password",
				message: "Password must contain at least one number",
				code: "MISSING_NUMBER",
			});
		}

		// Check for at least one special character
		if (!/[!@#$%^&*()_+=[\]{};':"\\|,.<>/?-]/.test(password)) {
			errors.push({
				field: "password",
				message: "Password must contain at least one special character",
				code: "MISSING_SPECIAL",
			});
		}

		return errors;
	}
}

/**
 * Specific validation functions for different request types
 */

/**
 * Validate user creation request
 */
export function validateCreateUserRequest(
	request: CreateUserRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate username
	errors.push(...Validator.validateString(request.username, "username", 3, 50));

	// Validate email (optional)
	if (request.email) {
		errors.push(...Validator.validateEmail(request.email));
	}

	// Validate password
	errors.push(...Validator.validatePassword(request.password));

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate account creation request
 */
export function validateCreateAccountRequest(
	request: CreateAccountRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate account_type_id
	errors.push(
		...Validator.validateUUID(request.account_type_id, "account_type_id"),
	);

	// Validate name
	errors.push(...Validator.validateString(request.name, "name", 1, 100));

	// Validate currency
	errors.push(...Validator.validateCurrency(request.currency));

	// Validate initial balance (optional)
	if (request.balance !== undefined) {
		errors.push(...Validator.validateAmount(request.balance, "balance", true)); // Allow negative for credit accounts
	}

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate category creation request
 */
export function validateCreateCategoryRequest(
	request: CreateCategoryRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate name
	errors.push(...Validator.validateString(request.name, "name", 1, 100));

	// Validate description (optional)
	if (request.description) {
		errors.push(
			...Validator.validateString(
				request.description,
				"description",
				0,
				500,
				false,
			),
		);
	}

	// Validate parent_category_id (optional)
	if (request.parent_category_id) {
		errors.push(
			...Validator.validateUUID(
				request.parent_category_id,
				"parent_category_id",
			),
		);
	}

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate transaction creation request
 */
export function validateCreateTransactionRequest(
	request: CreateTransactionRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate account_id
	errors.push(...Validator.validateUUID(request.account_id, "account_id"));

	// Validate category_id (optional)
	if (request.category_id) {
		errors.push(...Validator.validateUUID(request.category_id, "category_id"));
	}

	// Validate amount
	errors.push(...Validator.validateAmount(request.amount, "amount", true)); // Allow negative for corrections

	// Validate description
	errors.push(
		...Validator.validateString(request.description, "description", 1, 255),
	);

	// Validate transaction_date
	errors.push(
		...Validator.validateDateTime(request.transaction_date, "transaction_date"),
	);

	// Validate transaction_type
	const validTypes: TransactionType[] = ["income", "expense", "transfer"];
	if (!validTypes.includes(request.transaction_type)) {
		errors.push({
			field: "transaction_type",
			message: "Invalid transaction type",
			code: "INVALID_VALUE",
		});
	}

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate transfer creation request
 */
export function validateCreateTransferRequest(
	request: CreateTransferRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate from_account_id
	errors.push(
		...Validator.validateUUID(request.from_account_id, "from_account_id"),
	);

	// Validate to_account_id
	errors.push(
		...Validator.validateUUID(request.to_account_id, "to_account_id"),
	);

	// Validate that accounts are different
	if (request.from_account_id === request.to_account_id) {
		errors.push({
			field: "to_account_id",
			message: "Cannot transfer to the same account",
			code: "SAME_ACCOUNT",
		});
	}

	// Validate amount (must be positive for transfers)
	errors.push(
		...Validator.validateAmount(request.amount, "amount", false, false),
	);

	// Validate description
	errors.push(
		...Validator.validateString(request.description, "description", 1, 255),
	);

	// Validate transfer_date
	errors.push(
		...Validator.validateDateTime(request.transfer_date, "transfer_date"),
	);

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate budget creation request
 */
export function validateCreateBudgetRequest(
	request: CreateBudgetRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate budget_period_id
	errors.push(
		...Validator.validateUUID(request.budget_period_id, "budget_period_id"),
	);

	// Validate category_id
	errors.push(...Validator.validateUUID(request.category_id, "category_id"));

	// Validate allocated_amount (must be positive)
	errors.push(
		...Validator.validateAmount(
			request.allocated_amount,
			"allocated_amount",
			false,
			false,
		),
	);

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Validate goal creation request
 */
export function validateCreateGoalRequest(
	request: CreateGoalRequest,
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate user_id
	errors.push(...Validator.validateUUID(request.user_id, "user_id"));

	// Validate name
	errors.push(...Validator.validateString(request.name, "name", 1, 100));

	// Validate description (optional)
	if (request.description) {
		errors.push(
			...Validator.validateString(
				request.description,
				"description",
				0,
				500,
				false,
			),
		);
	}

	// Validate target_amount (must be positive)
	errors.push(
		...Validator.validateAmount(
			request.target_amount,
			"target_amount",
			false,
			false,
		),
	);

	// Validate target_date (optional)
	if (request.target_date) {
		errors.push(
			...Validator.validateDate(request.target_date, "target_date", false),
		);
	}

	// Validate priority (optional, 1-5)
	if (request.priority !== undefined) {
		if (
			typeof request.priority !== "number" ||
			request.priority < 1 ||
			request.priority > 5
		) {
			errors.push({
				field: "priority",
				message: "Priority must be between 1 and 5",
				code: "INVALID_RANGE",
			});
		}
	}

	return {
		isValid: errors.length === 0,
		errors,
	};
}

/**
 * Security utilities for client-side protection
 */
export namespace SecurityUtils {
	/**
	 * Sanitize string input to prevent XSS
	 */
	export function sanitizeString(input: string): string {
		if (!input) return "";

		return input
			.replace(/[<>]/g, "") // Remove angle brackets
			.replace(/javascript:/gi, "") // Remove javascript: protocol
			.replace(/on\w+=/gi, "") // Remove event handlers
			.trim();
	}

	/**
	 * Validate sort field against whitelist
	 */
	export function validateSortField(
		field: string,
		allowedFields: string[],
	): boolean {
		return allowedFields.includes(field);
	}

	/**
	 * Validate sort direction
	 */
	export function validateSortDirection(direction: string): boolean {
		return ["ASC", "DESC", "asc", "desc"].includes(direction);
	}

	/**
	 * Validate transaction filters
	 */
	export function validateTransactionFilters(filters: {
		account_id?: string;
		category_id?: string;
		start_date?: string;
		end_date?: string;
		min_amount?: number;
		max_amount?: number;
		search?: string;
		sort_by?: string;
		sort_direction?: string;
		limit?: number;
		offset?: number;
	}): string[] {
		const errors: string[] = [];

		// Validate UUIDs
		if (
			filters.account_id &&
			Validator.validateUUID(filters.account_id, "account_id").length > 0
		) {
			errors.push("Invalid account ID format");
		}

		if (
			filters.category_id &&
			Validator.validateUUID(filters.category_id, "category_id").length > 0
		) {
			errors.push("Invalid category ID format");
		}

		// Validate dates
		if (
			filters.start_date &&
			Validator.validateDate(filters.start_date, "start_date", false).length > 0
		) {
			errors.push("Invalid start date format");
		}

		if (
			filters.end_date &&
			Validator.validateDate(filters.end_date, "end_date", false).length > 0
		) {
			errors.push("Invalid end date format");
		}

		if (
			filters.start_date &&
			filters.end_date &&
			new Date(filters.start_date) > new Date(filters.end_date)
		) {
			errors.push("Start date must be before end date");
		}

		// Validate amounts
		if (
			filters.min_amount !== undefined &&
			(filters.min_amount < 0 || !Number.isFinite(filters.min_amount))
		) {
			errors.push("Minimum amount must be a positive number");
		}

		if (
			filters.max_amount !== undefined &&
			(filters.max_amount < 0 || !Number.isFinite(filters.max_amount))
		) {
			errors.push("Maximum amount must be a positive number");
		}

		if (
			filters.min_amount !== undefined &&
			filters.max_amount !== undefined &&
			filters.min_amount > filters.max_amount
		) {
			errors.push("Minimum amount must be less than maximum amount");
		}

		// Validate search
		if (filters.search && filters.search.length > 255) {
			errors.push("Search query is too long (max 255 characters)");
		}

		// Validate sort fields
		const allowedSortFields = [
			"amount",
			"description",
			"transaction_date",
			"created_at",
			"updated_at",
		];

		if (
			filters.sort_by &&
			!validateSortField(filters.sort_by, allowedSortFields)
		) {
			errors.push("Invalid sort field");
		}

		if (
			filters.sort_direction &&
			!validateSortDirection(filters.sort_direction)
		) {
			errors.push("Invalid sort direction");
		}

		// Validate pagination
		if (
			filters.limit !== undefined &&
			(filters.limit < 1 || filters.limit > 1000)
		) {
			errors.push("Limit must be between 1 and 1000");
		}

		if (filters.offset !== undefined && filters.offset < 0) {
			errors.push("Offset must be non-negative");
		}

		return errors;
	}

	/**
	 * Validate bulk transaction request
	 */
	export function validateBulkTransactionRequest(request: {
		transaction_ids: string[];
		action: unknown;
	}): string[] {
		const errors: string[] = [];

		if (!request.transaction_ids || request.transaction_ids.length === 0) {
			errors.push("No transaction IDs provided");
		}

		if (request.transaction_ids && request.transaction_ids.length > 100) {
			errors.push("Cannot process more than 100 transactions at once");
		}

		// Validate all transaction IDs
		for (const id of request.transaction_ids || []) {
			if (Validator.validateUUID(id, "transaction_id").length > 0) {
				errors.push(`Invalid transaction ID format: ${id}`);
				break; // Don't spam with all invalid IDs
			}
		}

		// Validate action
		if (!request.action || typeof request.action !== "object") {
			errors.push("Invalid action specified");
		}

		return errors;
	}

	/**
	 * Sanitize search query
	 */
	export function sanitizeSearchQuery(query: string): string {
		if (!query) return "";

		return query
			.replace(/[<>]/g, "") // Remove angle brackets
			.replace(/['"]/g, "") // Remove quotes
			.replace(/[;]/g, "") // Remove semicolons
			.trim()
			.substring(0, 100); // Limit length
	}
}

/**
 * Form validation hook helper
 */
export function createFormValidator<T>(
	validationFn: (data: T) => ValidationResult,
) {
	return (data: T) => {
		const result = validationFn(data);

		// Convert errors to a more convenient format for forms
		const fieldErrors: Record<string, string> = {};
		result.errors.forEach((error) => {
			if (!fieldErrors[error.field]) {
				fieldErrors[error.field] = error.message;
			}
		});

		return {
			isValid: result.isValid,
			errors: result.errors,
			fieldErrors,
		};
	};
}
