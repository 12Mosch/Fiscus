/**
 * React hooks for database operations
 * Provides easy-to-use hooks for common database operations with error handling
 */

import { useCallback, useEffect, useState } from "react";
import { DatabaseError, databaseService } from "./index";
import type {
	Account,
	AccountFilters,
	AccountWithType,
	CreateAccountInput,
	CreateTransactionInput,
	DashboardSummary,
	QueryOptions,
	QueryResult,
	Transaction,
	TransactionFilters,
	TransactionWithDetails,
	UpdateAccountInput,
	UpdateTransactionInput,
} from "./types";

// Generic hook state interface
interface UseAsyncState<T> {
	data: T | null;
	loading: boolean;
	error: string | null;
}

// Hook for managing async database operations
export function useAsyncOperation<T>() {
	const [state, setState] = useState<UseAsyncState<T>>({
		data: null,
		loading: false,
		error: null,
	});

	const execute = useCallback(async (operation: () => Promise<T>) => {
		setState((prev) => ({ ...prev, loading: true, error: null }));

		try {
			const result = await operation();
			setState({ data: result, loading: false, error: null });
			return result;
		} catch (error) {
			const errorMessage =
				error instanceof DatabaseError
					? error.message
					: "An unexpected error occurred";

			setState((prev) => ({ ...prev, loading: false, error: errorMessage }));
			throw error;
		}
	}, []);

	const reset = useCallback(() => {
		setState({ data: null, loading: false, error: null });
	}, []);

	return { ...state, execute, reset };
}

// Hook for database connection status
export function useDatabaseStatus() {
	const [status, setStatus] = useState<{
		connected: boolean;
		version: number;
		loading: boolean;
		error: string | null;
	}>({
		connected: false,
		version: 0,
		loading: true,
		error: null,
	});

	useEffect(() => {
		let mounted = true;

		const checkStatus = async () => {
			try {
				const health = await databaseService.getHealthStatus();
				if (mounted) {
					setStatus({
						connected: health.connected,
						version: health.version,
						loading: false,
						error: null,
					});
				}
			} catch (error) {
				if (mounted) {
					setStatus({
						connected: false,
						version: 0,
						loading: false,
						error: error instanceof Error ? error.message : "Unknown error",
					});
				}
			}
		};

		checkStatus();

		// Check status every 30 seconds
		const interval = setInterval(checkStatus, 30000);

		return () => {
			mounted = false;
			clearInterval(interval);
		};
	}, []);

	return status;
}

// Hook for fetching accounts
export function useAccounts(
	userId: string,
	filters?: AccountFilters,
	options?: QueryOptions,
) {
	const { data, loading, error, execute, reset } =
		useAsyncOperation<QueryResult<AccountWithType>>();

	const fetchAccounts = useCallback(async (): Promise<
		QueryResult<AccountWithType>
	> => {
		if (!userId) {
			return { data: [], total: 0, page: 1, limit: 50 };
		}
		return databaseService.accounts.findWithType(userId, filters, options);
	}, [userId, filters, options]);

	useEffect(() => {
		if (userId) {
			execute(fetchAccounts);
		}
	}, [userId, execute, fetchAccounts]);

	const refetch = useCallback(() => {
		if (userId) {
			execute(fetchAccounts);
		}
	}, [userId, execute, fetchAccounts]);

	return {
		accounts: data?.data || [],
		total: data?.total || 0,
		page: data?.page || 1,
		limit: data?.limit || 50,
		loading,
		error,
		refetch,
		reset,
	};
}

// Hook for fetching transactions
export function useTransactions(
	userId: string,
	filters?: TransactionFilters,
	options?: QueryOptions,
) {
	const { data, loading, error, execute, reset } =
		useAsyncOperation<QueryResult<TransactionWithDetails>>();

	const fetchTransactions = useCallback(async (): Promise<
		QueryResult<TransactionWithDetails>
	> => {
		if (!userId) {
			return { data: [], total: 0, page: 1, limit: 50 };
		}
		return databaseService.transactions.findWithDetails(
			userId,
			filters,
			options,
		);
	}, [userId, filters, options]);

	useEffect(() => {
		if (userId) {
			execute(fetchTransactions);
		}
	}, [userId, execute, fetchTransactions]);

	const refetch = useCallback(() => {
		if (userId) {
			execute(fetchTransactions);
		}
	}, [userId, execute, fetchTransactions]);

	return {
		transactions: data?.data || [],
		total: data?.total || 0,
		page: data?.page || 1,
		limit: data?.limit || 50,
		loading,
		error,
		refetch,
		reset,
	};
}

// Hook for account operations
export function useAccountOperations() {
	const { execute, loading, error } = useAsyncOperation<Account>();

	const createAccount = useCallback(
		async (accountData: CreateAccountInput) => {
			return execute(() => databaseService.accounts.create(accountData));
		},
		[execute],
	);

	const updateAccount = useCallback(
		async (id: string, updates: UpdateAccountInput) => {
			return execute(() => databaseService.accounts.update(id, updates));
		},
		[execute],
	);

	const deleteAccount = useCallback(
		async (id: string) => {
			return execute(() =>
				databaseService.accounts.delete(id).then(() => ({ id }) as Account),
			);
		},
		[execute],
	);

	const updateBalance = useCallback(
		async (id: string, balance: number) => {
			return execute(() => databaseService.accounts.updateBalance(id, balance));
		},
		[execute],
	);

	return {
		createAccount,
		updateAccount,
		deleteAccount,
		updateBalance,
		loading,
		error,
	};
}

// Hook for transaction operations
export function useTransactionOperations() {
	const { execute, loading, error } = useAsyncOperation<Transaction>();

	const createTransaction = useCallback(
		async (transactionData: CreateTransactionInput) => {
			return execute(() =>
				databaseService.transactions.createWithBalanceUpdate(transactionData),
			);
		},
		[execute],
	);

	const updateTransaction = useCallback(
		async (id: string, updates: UpdateTransactionInput) => {
			return execute(() => databaseService.transactions.update(id, updates));
		},
		[execute],
	);

	const deleteTransaction = useCallback(
		async (id: string) => {
			return execute(() =>
				databaseService.transactions
					.delete(id)
					.then(() => ({ id }) as Transaction),
			);
		},
		[execute],
	);

	return {
		createTransaction,
		updateTransaction,
		deleteTransaction,
		loading,
		error,
	};
}

// Hook for dashboard data
export function useDashboard(userId: string) {
	const { data, loading, error, execute, reset } =
		useAsyncOperation<DashboardSummary>();

	const fetchDashboard = useCallback(async (): Promise<DashboardSummary> => {
		if (!userId) {
			return {
				total_assets: 0,
				total_liabilities: 0,
				net_worth: 0,
				monthly_income: 0,
				monthly_expenses: 0,
				recent_transactions: [],
				account_balances: [],
				top_categories: [],
			};
		}

		// Fetch all dashboard data in parallel
		const [
			totalAssets,
			totalLiabilities,
			accountBalances,
			recentTransactions,
			categorySpending,
		] = await Promise.all([
			databaseService.accounts.getTotalAssets(userId),
			databaseService.accounts.getTotalLiabilities(userId),
			databaseService.accounts.getAccountBalances(userId),
			databaseService.transactions.getRecent(userId, 10),
			databaseService.transactions.getCategorySpending(userId),
		]);

		const netWorth = totalAssets - totalLiabilities;

		// Get current month income and expenses
		const now = new Date();
		const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
		const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);

		const [monthlyIncome, monthlyExpenses] = await Promise.all([
			databaseService.transactions.getTotalIncome(
				userId,
				startOfMonth.toISOString().split("T")[0],
				endOfMonth.toISOString().split("T")[0],
			),
			databaseService.transactions.getTotalExpenses(
				userId,
				startOfMonth.toISOString().split("T")[0],
				endOfMonth.toISOString().split("T")[0],
			),
		]);

		return {
			total_assets: totalAssets,
			total_liabilities: totalLiabilities,
			net_worth: netWorth,
			monthly_income: monthlyIncome,
			monthly_expenses: monthlyExpenses,
			recent_transactions: recentTransactions,
			account_balances: accountBalances,
			top_categories: categorySpending.slice(0, 5), // Top 5 categories
		};
	}, [userId]);

	useEffect(() => {
		if (userId) {
			execute(fetchDashboard);
		}
	}, [userId, execute, fetchDashboard]);

	const refetch = useCallback(() => {
		if (userId) {
			execute(fetchDashboard);
		}
	}, [userId, execute, fetchDashboard]);

	return {
		dashboard: data,
		loading,
		error,
		refetch,
		reset,
	};
}

// Hook for initializing database on app start
export function useDatabaseInitialization() {
	const [initialized, setInitialized] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		let mounted = true;

		const initialize = async () => {
			try {
				await databaseService.initialize();
				if (mounted) {
					setInitialized(true);
					setError(null);
				}
			} catch (error) {
				if (mounted) {
					setError(
						error instanceof Error
							? error.message
							: "Failed to initialize database",
					);
				}
			}
		};

		initialize();

		return () => {
			mounted = false;
		};
	}, []);

	return { initialized, error };
}
