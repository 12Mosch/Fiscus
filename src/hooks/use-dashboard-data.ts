/**
 * Custom hooks for fetching dashboard data from the Tauri API
 * Replaces mock data with real API calls while maintaining the same interface
 */

import { useCallback, useMemo } from "react";
import { apiClient } from "@/api/client";
import { useUserId } from "@/stores/auth-store";
import type {
	AccountFilters,
	BudgetFilters,
	ReportData,
	TransactionFilters,
} from "@/types/api";
import type {
	ChartDataPoint,
	DashboardStats,
	SpendingCategory,
} from "@/types/dashboard";
import {
	adaptApiAccountsToDashboard,
	adaptApiBudgetsToDashboard,
	adaptApiTransactionsToDashboard,
} from "@/utils/type-adapters";
import { useApiCall, useMultipleApiCalls } from "./use-api-call";

/**
 * Helper function to safely extract string values from ReportData
 */
function getStringValue(
	data: ReportData,
	key: string,
	fallback: string,
): string {
	const value = data[key];
	if (typeof value === "string") return value;
	if (typeof value === "number") return value.toString();
	return fallback;
}

/**
 * Helper function to safely extract number values from ReportData
 */
function getNumberValue(
	data: ReportData,
	key: string,
	fallback: number,
): number {
	const value = data[key];
	if (typeof value === "number") return value;
	if (typeof value === "string") {
		const parsed = parseFloat(value);
		return Number.isNaN(parsed) ? fallback : parsed;
	}
	return fallback;
}

/**
 * Hook for fetching user accounts
 */
export function useAccounts() {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			const filters: AccountFilters = {
				user_id: userId,
			};

			const apiAccounts = await apiClient.getAccounts(filters);
			return adaptApiAccountsToDashboard(apiAccounts);
		}, [userId]),
		{ immediate: true },
	);

	return {
		accounts: data || [],
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for fetching recent transactions
 */
export function useRecentTransactions(limit: number = 10) {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			const filters: TransactionFilters = {
				user_id: userId,
				limit,
				sort_by: "transaction_date",
				sort_direction: "DESC",
			};

			const apiTransactions = await apiClient.getTransactions(filters);
			return adaptApiTransactionsToDashboard(apiTransactions);
		}, [userId, limit]),
		{ immediate: true },
	);

	return {
		transactions: data || [],
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for fetching user budgets
 */
export function useBudgets() {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			const filters: BudgetFilters = {
				user_id: userId,
			};

			const apiBudgets = await apiClient.getBudgets(filters);
			return adaptApiBudgetsToDashboard(apiBudgets);
		}, [userId]),
		{ immediate: true },
	);

	return {
		budgets: data || [],
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for fetching account balance history
 */
export function useAccountBalanceHistory(days: number = 30) {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			return await apiClient.getAccountBalanceHistory(userId, undefined, days);
		}, [userId, days]),
		{ immediate: true },
	);

	// Transform API response to ChartDataPoint format
	const chartData = useMemo((): ChartDataPoint[] => {
		if (!data) return [];

		return data.map((item) => {
			const date = getStringValue(
				item,
				"date",
				new Date().toISOString().split("T")[0],
			);
			const amount = getNumberValue(item, "amount", 0);

			return {
				date,
				value: amount,
				label: `Balance: $${amount.toFixed(2)}`,
			};
		});
	}, [data]);

	return {
		balanceHistory: chartData,
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for fetching spending by category
 */
export function useSpendingByCategory(
	period: "monthly" | "yearly" = "monthly",
) {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			// Calculate date range based on period
			const endDate = new Date();
			const startDate = new Date();

			if (period === "monthly") {
				startDate.setMonth(endDate.getMonth() - 1);
			} else {
				startDate.setFullYear(endDate.getFullYear() - 1);
			}

			return await apiClient.getSpendingByCategory(
				userId,
				startDate.toISOString().split("T")[0],
				endDate.toISOString().split("T")[0],
			);
		}, [userId, period]),
		{ immediate: true },
	);

	// Transform API response to SpendingCategory format
	const spendingCategories = useMemo((): SpendingCategory[] => {
		if (!data) return [];

		const total = data.reduce((sum, item) => {
			const amount = getNumberValue(item, "amount", 0);
			return sum + Math.abs(amount);
		}, 0);

		return data.map((item, index) => {
			const category = getStringValue(
				item,
				"category",
				`Category ${index + 1}`,
			);
			const amount = Math.abs(getNumberValue(item, "amount", 0));
			const color = getStringValue(
				item,
				"color",
				`hsl(${(index * 137.5) % 360}, 70%, 50%)`,
			);
			const transactions = getNumberValue(item, "count", 0);

			return {
				category,
				amount,
				percentage: total > 0 ? (amount / total) * 100 : 0,
				color,
				transactions,
			};
		});
	}, [data]);

	return {
		spendingCategories,
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for fetching monthly spending trends
 */
export function useMonthlySpendingTrend(months: number = 6) {
	const userId = useUserId();

	const { data, loading, error, execute } = useApiCall(
		useCallback(async () => {
			if (!userId) return [];

			return await apiClient.getMonthlySpendingTrend(userId, months);
		}, [userId, months]),
		{ immediate: true },
	);

	// Transform API response to ChartDataPoint format for income/expense comparison
	const chartData = useMemo((): ChartDataPoint[] => {
		if (!data) return [];

		const result: ChartDataPoint[] = [];

		data.forEach((item, index) => {
			const date = getStringValue(item, "date", `Month ${index + 1}`);
			const income = getNumberValue(item, "income", 0);
			const expenses = getNumberValue(item, "expenses", 0);

			// Add income data point
			result.push({
				date,
				value: income,
				label: "Income",
			});

			// Add expense data point
			result.push({
				date,
				value: Math.abs(expenses),
				label: "Expenses",
			});
		});

		return result;
	}, [data]);

	return {
		monthlyTrend: chartData,
		loading,
		error,
		refetch: execute,
	};
}

/**
 * Hook for calculating dashboard statistics from real data
 */
export function useDashboardStats() {
	const { accounts } = useAccounts();
	const { transactions } = useRecentTransactions(100); // Get more transactions for calculations

	// Calculate dashboard stats from real data
	const dashboardStats = useMemo((): DashboardStats => {
		if (!accounts.length) {
			return {
				totalBalance: 0,
				monthlyIncome: 0,
				monthlyExpenses: 0,
				savingsRate: 0,
				currency: "USD",
			};
		}

		// Calculate total balance from all accounts
		const totalBalance = accounts.reduce(
			(sum, account) => sum + account.balance,
			0,
		);

		// Calculate monthly income and expenses from recent transactions
		const currentMonth = new Date().getMonth();
		const currentYear = new Date().getFullYear();

		const monthlyTransactions = transactions.filter((tx) => {
			const txDate = new Date(tx.date);
			return (
				txDate.getMonth() === currentMonth &&
				txDate.getFullYear() === currentYear
			);
		});

		const monthlyIncome = monthlyTransactions
			.filter((tx) => tx.type === "income")
			.reduce((sum, tx) => sum + Math.abs(tx.amount), 0);

		const monthlyExpenses = monthlyTransactions
			.filter((tx) => tx.type === "expense")
			.reduce((sum, tx) => sum + Math.abs(tx.amount), 0);

		// Calculate savings rate
		const savingsRate =
			monthlyIncome > 0
				? ((monthlyIncome - monthlyExpenses) / monthlyIncome) * 100
				: 0;

		return {
			totalBalance,
			monthlyIncome,
			monthlyExpenses,
			savingsRate: Math.max(0, savingsRate), // Ensure non-negative
			currency: accounts[0]?.currency || "USD",
		};
	}, [accounts, transactions]);

	return {
		dashboardStats,
		loading: false, // Calculated from other data
		error: null,
	};
}

/**
 * Hook for fetching all dashboard data at once
 */
export function useAllDashboardData() {
	const userId = useUserId();

	const apiCalls = useMemo(() => {
		if (!userId) return {};

		return {
			accounts: () => apiClient.getAccounts({ user_id: userId }),
			recentTransactions: () =>
				apiClient.getTransactions({
					user_id: userId,
					limit: 10,
					sort_by: "transaction_date",
					sort_direction: "DESC",
				}),
			budgets: () => apiClient.getBudgets({ user_id: userId }),
			balanceHistory: () =>
				apiClient.getAccountBalanceHistory(userId, undefined, 30),
			spendingByCategory: () => {
				const endDate = new Date();
				const startDate = new Date();
				startDate.setMonth(endDate.getMonth() - 1);

				return apiClient.getSpendingByCategory(
					userId,
					startDate.toISOString().split("T")[0],
					endDate.toISOString().split("T")[0],
				);
			},
		};
	}, [userId]);

	const { data, loading, error, execute } = useMultipleApiCalls(apiCalls, {
		immediate: true,
	});

	return {
		data: data || {},
		loading,
		error,
		refetch: execute,
	};
}
