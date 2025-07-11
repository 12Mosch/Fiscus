/**
 * Centralized exports for all Zustand stores
 * Provides easy access to all stores and their hooks
 */

// Accounts store
export * from "./accounts-store";

// Authentication store
export * from "./auth-store";
// Budgets store
export * from "./budgets-store";
// Categories store
export * from "./categories-store";
// Goals store
export * from "./goals-store";
// Reports store
export * from "./reports-store";
// Theme store
export * from "./theme-store";
// Transactions store
export * from "./transactions-store";

/**
 * Store cleanup functions for proper resource management
 */
import { cleanupThemeStore } from "./theme-store";

export const cleanupAllStores = () => {
	cleanupThemeStore();
	// Add other cleanup functions as needed
};

import { useAccountsStore } from "./accounts-store";
/**
 * Reset all stores to initial state
 * Useful for logout or testing scenarios
 */
import { useAuthStore } from "./auth-store";
import { useBudgetsStore } from "./budgets-store";
import { useCategoriesStore } from "./categories-store";
import { useGoalsStore } from "./goals-store";
import { useReportsStore } from "./reports-store";
import { useTransactionsStore } from "./transactions-store";

export const resetAllStores = () => {
	useAuthStore.getState().logout();
	useAccountsStore.getState().reset();
	useTransactionsStore.getState().reset();
	useCategoriesStore.getState().reset();
	useBudgetsStore.getState().reset();
	useGoalsStore.getState().reset();
	useReportsStore.getState().reset();
};

/**
 * Initialize all stores
 * Call this on app startup
 */
export const initializeStores = async () => {
	// Initialize auth store
	await useAuthStore.getState().initialize();

	// Other stores will be initialized as needed when data is loaded
};
