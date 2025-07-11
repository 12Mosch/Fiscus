/**
 * React hook for database seeding in development
 * Provides utilities to seed the database from within the React application
 */

import { useCallback, useState } from "react";
import { clearDatabase, type SeedOptions, seedDatabase } from "./index";

export interface SeedingState {
	isSeeding: boolean;
	isClearing: boolean;
	error: string | null;
	lastSeeded: Date | null;
}

export interface SeedingActions {
	seed: (options?: SeedOptions) => Promise<void>;
	clear: () => Promise<void>;
	seedWithClear: (options?: SeedOptions) => Promise<void>;
	clearError: () => void;
}

/**
 * Hook for managing database seeding operations
 * @returns Object containing seeding state and actions
 */
export function useSeeding(): SeedingState & SeedingActions {
	const [state, setState] = useState<SeedingState>({
		isSeeding: false,
		isClearing: false,
		error: null,
		lastSeeded: null,
	});

	const clearError = useCallback(() => {
		setState((prev) => ({ ...prev, error: null }));
	}, []);

	/**
	 * Seeds the database with sample data.
	 *
	 * On error, this function updates the hook state by setting `state.error`
	 * and then rethrows the error. Callers must catch these errors to avoid
	 * unhandled promise rejections.
	 *
	 * @param options - Optional seeding configuration
	 * @throws {Error} The original error after updating hook state
	 */
	const seed = useCallback(async (options?: SeedOptions) => {
		setState((prev) => ({ ...prev, isSeeding: true, error: null }));

		try {
			await seedDatabase(options);
			setState((prev) => ({
				...prev,
				isSeeding: false,
				lastSeeded: new Date(),
			}));
		} catch (error) {
			setState((prev) => ({
				...prev,
				isSeeding: false,
				error: error instanceof Error ? error.message : "Unknown seeding error",
			}));
			throw error;
		}
	}, []);

	/**
	 * Clears all data from the database.
	 *
	 * On error, this function updates the hook state by setting `state.error`
	 * and then rethrows the error. Callers must catch these errors to avoid
	 * unhandled promise rejections.
	 *
	 * @throws {Error} The original error after updating hook state
	 */
	const clear = useCallback(async () => {
		setState((prev) => ({ ...prev, isClearing: true, error: null }));

		try {
			await clearDatabase();
			setState((prev) => ({
				...prev,
				isClearing: false,
			}));
		} catch (error) {
			setState((prev) => ({
				...prev,
				isClearing: false,
				error:
					error instanceof Error ? error.message : "Unknown clearing error",
			}));
			throw error;
		}
	}, []);

	const seedWithClear = useCallback(
		async (options?: SeedOptions) => {
			const finalOptions = { ...options, clearExisting: true };
			await seed(finalOptions);
		},
		[seed],
	);

	return {
		...state,
		seed,
		clear,
		seedWithClear,
		clearError,
	};
}

/**
 * Predefined seeding configurations for common use cases
 */
export const SEEDING_PRESETS: Record<string, SeedOptions> = {
	full: {
		clearExisting: true,
		includeUsers: true,
		includeAccounts: true,
		includeCategories: true,
		includeTransactions: true,
		includeBudgets: true,
		includeGoals: true,
		transactionsPerAccount: 25,
	},
	minimal: {
		clearExisting: false,
		includeUsers: true,
		includeAccounts: true,
		includeCategories: true,
		includeTransactions: false,
		includeBudgets: false,
		includeGoals: false,
	},
	demo: {
		clearExisting: true,
		includeUsers: true,
		includeAccounts: true,
		includeCategories: true,
		includeTransactions: true,
		includeBudgets: true,
		includeGoals: true,
		transactionsPerAccount: 15,
	},
	testing: {
		clearExisting: true,
		includeUsers: true,
		includeAccounts: true,
		includeCategories: true,
		includeTransactions: true,
		includeBudgets: false,
		includeGoals: false,
		transactionsPerAccount: 5,
	},
};
