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
		if (oldValue === 0 && newValue === 0) return 0;
		if (oldValue === 0) return newValue > 0 ? Infinity : -Infinity;
		return ((newValue - oldValue) / Math.abs(oldValue)) * 100;
	},

	/**
	 * Get date range for common periods
	 * Handles edge cases like month-end dates and timezone considerations
	 */
	getDateRange(period: "today" | "week" | "month" | "quarter" | "year"): {
		start: string;
		end: string;
	} {
		// Helper function to format date in local timezone as YYYY-MM-DD
		const formatLocalDate = (date: Date): string => {
			const year = date.getFullYear();
			const month = String(date.getMonth() + 1).padStart(2, "0");
			const day = String(date.getDate()).padStart(2, "0");
			return `${year}-${month}-${day}`;
		};

		// Use local timezone for consistent date calculations
		const now = new Date();
		const start = new Date();

		switch (period) {
			case "today": {
				// Start of today in local timezone
				start.setHours(0, 0, 0, 0);
				break;
			}
			case "week": {
				// Exactly 7 days ago from now
				start.setDate(now.getDate() - 7);
				start.setHours(0, 0, 0, 0);
				break;
			}
			case "month": {
				// Start of the same day in the previous month
				// Handle month-end edge cases properly
				const currentMonth = now.getMonth();
				const currentYear = now.getFullYear();
				const currentDay = now.getDate();

				// Go to previous month
				let targetMonth = currentMonth - 1;
				let targetYear = currentYear;

				if (targetMonth < 0) {
					targetMonth = 11;
					targetYear--;
				}

				// Get the last day of the target month
				const lastDayOfTargetMonth = new Date(
					targetYear,
					targetMonth + 1,
					0,
				).getDate();

				// Use the current day or the last day of target month, whichever is smaller
				const targetDay = Math.min(currentDay, lastDayOfTargetMonth);

				start.setFullYear(targetYear, targetMonth, targetDay);
				start.setHours(0, 0, 0, 0);
				break;
			}
			case "quarter": {
				// Start of the same day 3 months ago
				// Handle quarter-end edge cases properly
				const quarterCurrentMonth = now.getMonth();
				const quarterCurrentYear = now.getFullYear();
				const quarterCurrentDay = now.getDate();

				// Go back 3 months
				let quarterTargetMonth = quarterCurrentMonth - 3;
				let quarterTargetYear = quarterCurrentYear;

				if (quarterTargetMonth < 0) {
					quarterTargetMonth += 12;
					quarterTargetYear--;
				}

				// Get the last day of the target month
				const lastDayOfQuarterTargetMonth = new Date(
					quarterTargetYear,
					quarterTargetMonth + 1,
					0,
				).getDate();

				// Use the current day or the last day of target month, whichever is smaller
				const quarterTargetDay = Math.min(
					quarterCurrentDay,
					lastDayOfQuarterTargetMonth,
				);

				start.setFullYear(
					quarterTargetYear,
					quarterTargetMonth,
					quarterTargetDay,
				);
				start.setHours(0, 0, 0, 0);
				break;
			}
			case "year": {
				// Exactly one year ago from today
				const yearCurrentYear = now.getFullYear();
				const yearCurrentMonth = now.getMonth();
				const yearCurrentDay = now.getDate();

				// Handle leap year edge case (Feb 29 -> Feb 28)
				let yearTargetDay = yearCurrentDay;
				if (yearCurrentMonth === 1 && yearCurrentDay === 29) {
					// Feb 29 in a leap year
					const targetYear = yearCurrentYear - 1;
					const isTargetLeapYear =
						(targetYear % 4 === 0 && targetYear % 100 !== 0) ||
						targetYear % 400 === 0;
					if (!isTargetLeapYear) {
						yearTargetDay = 28; // Feb 28 in non-leap year
					}
				}

				start.setFullYear(yearCurrentYear - 1, yearCurrentMonth, yearTargetDay);
				start.setHours(0, 0, 0, 0);
				break;
			}
		}

		// Use local date formatting to avoid timezone issues
		return {
			start: formatLocalDate(start),
			end: formatLocalDate(now),
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
