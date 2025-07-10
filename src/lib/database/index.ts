/**
 * Database module exports
 * Central export point for all database-related functionality
 */

// Connection utilities
export {
	closeDatabase,
	DatabaseError,
	db,
	executeCommand,
	executeQuery,
	executeTransaction,
	formatDateForDb,
	formatDateTimeForDb,
	generateId,
	getDatabase,
	getDatabaseVersion,
	isDatabaseConnected,
} from "./connection";
export { AccountRepository } from "./repositories/accounts";
// Repository classes
export { BaseRepository } from "./repositories/base";
export { TransactionRepository } from "./repositories/transactions";
// Seeding utilities (development only)
export * from "./seeding";
// Type definitions
export type * from "./types";

// Import utilities for local use
import {
	closeDatabase,
	executeCommand,
	executeQuery,
	executeTransaction,
	formatDateForDb,
	formatDateTimeForDb,
	generateId,
	getDatabase,
	getDatabaseVersion,
	isDatabaseConnected,
} from "./connection";
// Repository instances (singletons)
import { AccountRepository } from "./repositories/accounts";
import { TransactionRepository } from "./repositories/transactions";

export const accountRepository = new AccountRepository();
export const transactionRepository = new TransactionRepository();

// Database service class for higher-level operations
export class DatabaseService {
	public accounts = accountRepository;
	public transactions = transactionRepository;

	/**
	 * Initialize the database service
	 * This should be called once when the application starts
	 */
	async initialize(): Promise<void> {
		try {
			const isConnected = await isDatabaseConnected();
			if (!isConnected) {
				throw new Error("Failed to establish database connection");
			}

			console.log("Database service initialized successfully");
		} catch (error) {
			console.error("Failed to initialize database service:", error);
			throw error;
		}
	}

	/**
	 * Close all database connections
	 * This should be called when the application is shutting down
	 */
	async shutdown(): Promise<void> {
		try {
			await closeDatabase();
			console.log("Database service shut down successfully");
		} catch (error) {
			console.error("Failed to shut down database service:", error);
			throw error;
		}
	}

	/**
	 * Get database health status
	 * @returns Database health information
	 */
	async getHealthStatus(): Promise<{
		connected: boolean;
		version: number;
		timestamp: string;
	}> {
		try {
			const [connected, version] = await Promise.all([
				isDatabaseConnected(),
				getDatabaseVersion(),
			]);

			return {
				connected,
				version,
				timestamp: new Date().toISOString(),
			};
		} catch (_error) {
			return {
				connected: false,
				version: 0,
				timestamp: new Date().toISOString(),
			};
		}
	}
}

// Export singleton database service instance
export const databaseService = new DatabaseService();

// Utility functions for common database operations
export const dbUtils = {
	/**
	 * Generate a new UUID for database records
	 */
	generateId,

	/**
	 * Format a date for SQLite storage
	 */
	formatDate: formatDateForDb,

	/**
	 * Format a datetime for SQLite storage
	 */
	formatDateTime: formatDateTimeForDb,

	/**
	 * Validate that a string is a valid UUID
	 */
	isValidId(id: string): boolean {
		const uuidRegex =
			/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
		return uuidRegex.test(id);
	},

	/**
	 * Sanitize a string for SQL LIKE queries
	 */
	sanitizeForLike(input: string): string {
		return input.replace(/[%_]/g, "\\$&");
	},

	/**
	 * Parse JSON tags from database
	 */
	parseTags(tagsJson?: string): string[] {
		if (!tagsJson) return [];
		try {
			return JSON.parse(tagsJson);
		} catch {
			return [];
		}
	},

	/**
	 * Stringify tags for database storage
	 */
	stringifyTags(tags: string[]): string {
		return JSON.stringify(tags);
	},

	/**
	 * Format currency amount for display
	 */
	formatCurrency(amount: number, currency: string = "USD"): string {
		return new Intl.NumberFormat("en-US", {
			style: "currency",
			currency: currency,
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(amount);
	},

	/**
	 * Calculate percentage change between two values
	 */
	calculatePercentageChange(oldValue: number, newValue: number): number {
		if (oldValue === 0) return newValue === 0 ? 0 : 100;
		return ((newValue - oldValue) / Math.abs(oldValue)) * 100;
	},

	/**
	 * Get date range for common periods
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
				start.setDate(now.getDate() - 7);
				break;
			case "month":
				start.setMonth(now.getMonth() - 1);
				break;
			case "quarter":
				start.setMonth(now.getMonth() - 3);
				break;
			case "year":
				start.setFullYear(now.getFullYear() - 1);
				break;
		}

		return {
			start: formatDateForDb(start),
			end: formatDateForDb(now),
		};
	},
};

// Export everything for convenience
export default {
	service: databaseService,
	repositories: {
		accounts: accountRepository,
		transactions: transactionRepository,
	},
	utils: dbUtils,
	connection: {
		getDatabase,
		closeDatabase,
		executeQuery,
		executeCommand,
		executeTransaction,
		isDatabaseConnected,
		getDatabaseVersion,
	},
};
