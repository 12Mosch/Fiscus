/**
 * React hooks for database operations
 * Provides easy-to-use hooks for common database operations with error handling
 *
 * SECURITY UPDATE: Now uses secure API service instead of direct database access
 */

import { useCallback, useEffect, useState } from "react";
import { FiscusApiError } from "@/api/client";
import type {
	Account,
	AccountFilters,
	CreateAccountRequest,
	CreateTransactionRequest,
	Transaction,
	TransactionFilters,
	UpdateAccountRequest,
	UpdateTransactionRequest,
} from "@/types/api";
import type {
	DashboardSummary,
	QueryOptions,
	QueryResult,
} from "../api-service";
import { apiService } from "../api-service";

// Generic hook state interface
interface UseAsyncState<T> {
	data: T | null;
	loading: boolean;
	error: string | null;
}

// Hook for managing async API operations
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
				error instanceof FiscusApiError
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

// Hook for API service status
export function useApiStatus() {
	const [status, setStatus] = useState<{
		connected: boolean;
		loading: boolean;
		error: string | null;
	}>({
		connected: false,
		loading: true,
		error: null,
	});

	useEffect(() => {
		let mounted = true;

		const checkStatus = async () => {
			try {
				// For API service, we can check if we can make a simple call
				// This is a placeholder - in a real implementation you might have a health check endpoint
				await apiService.initialize();
				if (mounted) {
					setStatus({
						connected: true,
						loading: false,
						error: null,
					});
				}
			} catch (error) {
				if (mounted) {
					setStatus({
						connected: false,
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
	_filters?: Omit<AccountFilters, "user_id">,
	options?: QueryOptions,
) {
	const { data, loading, error, execute, reset } =
		useAsyncOperation<Account[]>();

	const fetchAccounts = useCallback(async (): Promise<Account[]> => {
		if (!userId) {
			return [];
		}
		return apiService.accounts.findWithType(userId, options);
	}, [userId, options]);

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
		accounts: data || [],
		loading,
		error,
		refetch,
		reset,
	};
}

// Hook for fetching transactions
export function useTransactions(
	userId: string,
	filters?: Omit<TransactionFilters, "user_id">,
	options?: QueryOptions,
) {
	const { data, loading, error, execute, reset } =
		useAsyncOperation<QueryResult<Transaction>>();

	const fetchTransactions = useCallback(async (): Promise<
		QueryResult<Transaction>
	> => {
		if (!userId) {
			return { data: [], total: 0, page: 1, limit: 50 };
		}
		return apiService.transactions.findWithDetails(userId, filters, options);
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
	const {
		execute: executeDelete,
		loading: deleteLoading,
		error: deleteError,
	} = useAsyncOperation<boolean>();

	const createAccount = useCallback(
		async (accountData: CreateAccountRequest) => {
			return execute(() => apiService.accounts.create(accountData));
		},
		[execute],
	);

	const updateAccount = useCallback(
		async (
			id: string,
			userId: string,
			updates: Partial<UpdateAccountRequest>,
		) => {
			return execute(() => apiService.accounts.update(id, userId, updates));
		},
		[execute],
	);

	const deleteAccount = useCallback(
		async (id: string, userId: string) => {
			return executeDelete(() => apiService.accounts.delete(id, userId));
		},
		[executeDelete],
	);

	return {
		createAccount,
		updateAccount,
		deleteAccount,
		loading: loading || deleteLoading,
		error: error || deleteError,
	};
}

// Hook for transaction operations
export function useTransactionOperations() {
	const { execute, loading, error } = useAsyncOperation<Transaction>();
	const {
		execute: executeDelete,
		loading: deleteLoading,
		error: deleteError,
	} = useAsyncOperation<boolean>();

	const createTransaction = useCallback(
		async (transactionData: CreateTransactionRequest) => {
			return execute(() =>
				apiService.transactions.createWithBalanceUpdate(transactionData),
			);
		},
		[execute],
	);

	const updateTransaction = useCallback(
		async (
			id: string,
			userId: string,
			updates: Partial<UpdateTransactionRequest>,
		) => {
			return execute(() => apiService.transactions.update(id, userId, updates));
		},
		[execute],
	);

	const deleteTransaction = useCallback(
		async (id: string, userId: string) => {
			return executeDelete(() => apiService.transactions.delete(id, userId));
		},
		[executeDelete],
	);

	return {
		createTransaction,
		updateTransaction,
		deleteTransaction,
		loading: loading || deleteLoading,
		error: error || deleteError,
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

		// Use the API service's dashboard summary method
		return apiService.getDashboardSummary(userId);
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

// Hook for initializing API service on app start
export function useApiInitialization() {
	const [initialized, setInitialized] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		let mounted = true;

		const initialize = async () => {
			try {
				await apiService.initialize();
				if (mounted) {
					setInitialized(true);
					setError(null);
				}
			} catch (error) {
				if (mounted) {
					setError(
						error instanceof Error
							? error.message
							: "Failed to initialize API service",
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

// Legacy alias for backward compatibility
export const useDatabaseInitialization = useApiInitialization;
