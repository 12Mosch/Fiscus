/**
 * Database connection management for Fiscus
 * Handles SQLite database connections using Tauri SQL plugin
 */

import Database from "@tauri-apps/plugin-sql";

// Database configuration
const DATABASE_NAME = "sqlite:fiscus.db";

// Global database instance
let dbInstance: Database | null = null;

/**
 * Initialize and get database connection
 * This function ensures we have a single database connection throughout the app
 */
export async function getDatabase(): Promise<Database> {
	if (!dbInstance) {
		try {
			// Load the database with migrations applied automatically
			dbInstance = await Database.load(DATABASE_NAME);
			console.log("Database connection established successfully");
		} catch (error) {
			console.error("Failed to connect to database:", error);
			throw new DatabaseError(
				"Failed to connect to database",
				error instanceof Error
					? { message: error.message, stack: error.stack }
					: error,
			);
		}
	}
	return dbInstance;
}

/**
 * Close database connection
 * Should be called when the application is shutting down
 */
export async function closeDatabase(): Promise<void> {
	if (dbInstance) {
		try {
			await dbInstance.close();
			dbInstance = null;
			console.log("Database connection closed successfully");
		} catch (error) {
			console.error("Failed to close database connection:", error);
			throw new DatabaseError(
				"Failed to close database connection",
				error instanceof Error
					? { message: error.message, stack: error.stack }
					: error,
			);
		}
	}
}

/**
 * Execute a SQL query with parameters
 * @param query SQL query string
 * @param params Query parameters
 * @returns Query result
 */
export async function executeQuery<T = unknown>(
	query: string,
	params: unknown[] = [],
): Promise<T[]> {
	try {
		const db = await getDatabase();
		const result = await db.select<T[]>(query, params);
		return result;
	} catch (error) {
		console.error("Query execution failed:", { query, params, error });
		throw new DatabaseError(`Query execution failed: ${error}`, error);
	}
}

/**
 * Execute a SQL command (INSERT, UPDATE, DELETE)
 * @param query SQL command string
 * @param params Command parameters
 * @returns Command result with affected rows and last insert ID
 */
export async function executeCommand(
	query: string,
	params: unknown[] = [],
): Promise<{ rowsAffected: number; lastInsertId?: number }> {
	try {
		const db = await getDatabase();
		const result = await db.execute(query, params);
		return {
			rowsAffected: result.rowsAffected,
			lastInsertId: result.lastInsertId,
		};
	} catch (error) {
		console.error("Command execution failed:", { query, params, error });
		throw new DatabaseError(`Command execution failed: ${error}`, error);
	}
}

/**
 * Execute multiple SQL commands in a transaction
 * @param commands Array of SQL commands with parameters
 * @returns Array of command results
 */
export async function executeTransaction(
	commands: Array<{ query: string; params?: unknown[] }>,
): Promise<Array<{ rowsAffected: number; lastInsertId?: number }>> {
	const db = await getDatabase();

	try {
		// Begin transaction
		await db.execute("BEGIN TRANSACTION");

		const results = [];
		for (const command of commands) {
			const result = await db.execute(command.query, command.params || []);
			results.push({
				rowsAffected: result.rowsAffected,
				lastInsertId: result.lastInsertId,
			});
		}

		// Commit transaction
		await db.execute("COMMIT");
		return results;
	} catch (error) {
		// Rollback transaction on error
		let rollbackError: unknown = null;
		try {
			await db.execute("ROLLBACK");
		} catch (rollbackErr) {
			rollbackError = rollbackErr;
			console.error("Failed to rollback transaction:", rollbackErr);
		}

		console.error("Transaction failed:", { commands, error });

		// Combine original error and rollback error information
		const errorMessage = rollbackError
			? `Transaction failed: ${error}. Additionally, rollback failed: ${rollbackError}`
			: `Transaction failed: ${error}`;

		const errorDetails = rollbackError
			? { originalError: error, rollbackError }
			: error;

		throw new DatabaseError(errorMessage, errorDetails);
	}
}

/**
 * Check if database is connected and accessible
 * @returns Promise<boolean> indicating connection status
 */
export async function isDatabaseConnected(): Promise<boolean> {
	try {
		const db = await getDatabase();
		// Try a simple query to test connection
		await db.select("SELECT 1 as test");
		return true;
	} catch (error) {
		console.error("Database connection test failed:", error);
		return false;
	}
}

/**
 * Get database schema version for migration tracking
 * @returns Current schema version
 */
export async function getDatabaseVersion(): Promise<number> {
	try {
		const result = await executeQuery<{ version: number }>(
			"PRAGMA user_version",
		);
		return result[0]?.version || 0;
	} catch (error) {
		console.error("Failed to get database version:", error);
		return 0;
	}
}

/**
 * Custom error class for database operations
 */
export class DatabaseError extends Error {
	public code?: string;
	public details?: unknown;

	constructor(message: string, details?: unknown, code?: string) {
		super(message);
		this.name = "DatabaseError";
		this.code = code;
		this.details = details;
	}
}

/**
 * Utility function to generate UUID for database records
 * Uses crypto.randomUUID() if available, otherwise throws an error
 */
export function generateId(): string {
	if (typeof crypto !== "undefined" && crypto.randomUUID) {
		return crypto.randomUUID();
	}

	// For security-critical applications, require proper crypto support
	throw new Error(
		"Secure random UUID generation not available in this environment",
	);
}

/**
 * Utility function to format dates for SQLite
 * @param date Date object or ISO string
 * @returns Formatted date string for SQLite
 * @throws Error if the date is invalid
 */
export function formatDateForDb(date: Date | string): string {
	let dateObj: Date;

	if (date instanceof Date) {
		// Use existing Date object if it's already a Date instance
		dateObj = date;
	} else if (typeof date === "string") {
		// Validate string is not empty
		if (!date.trim()) {
			throw new Error("Date string cannot be empty");
		}
		dateObj = new Date(date);
	} else {
		throw new Error("Date must be a Date object or a valid date string");
	}

	// Check if the date is valid
	if (Number.isNaN(dateObj.getTime())) {
		throw new Error(`Invalid date: ${date}`);
	}

	return dateObj.toISOString().split("T")[0]; // YYYY-MM-DD format
}

/**
 * Utility function to format datetime for SQLite
 * @param date Date object or ISO string
 * @returns Formatted datetime string for SQLite
 * @throws Error if the date is invalid
 */
export function formatDateTimeForDb(date: Date | string): string {
	let dateObj: Date;

	if (date instanceof Date) {
		// Use existing Date object if it's already a Date instance
		dateObj = date;
	} else if (typeof date === "string") {
		// Validate string is not empty
		if (!date.trim()) {
			throw new Error("Date string cannot be empty");
		}
		dateObj = new Date(date);
	} else {
		throw new Error("Date must be a Date object or a valid date string");
	}

	// Check if the date is valid
	if (Number.isNaN(dateObj.getTime())) {
		throw new Error(`Invalid date: ${date}`);
	}

	return dateObj.toISOString(); // Full ISO string
}

// Export database instance getter for direct access when needed
export { getDatabase as db };
