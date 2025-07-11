/**
 * Reports store using Zustand
 * Manages financial reports and analytics data
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type { ReportData } from "../types/api";

interface ReportsState {
	/** Financial overview data */
	financialOverview: ReportData | null;
	/** Spending by category data */
	spendingByCategory: ReportData[] | null;
	/** Monthly spending trend data */
	monthlySpendingTrend: ReportData[] | null;
	/** Account balance history data */
	accountBalanceHistory: ReportData[] | null;
	/** Budget performance data */
	budgetPerformance: ReportData[] | null;
	/** Net worth progression data */
	netWorthProgression: ReportData[] | null;
	/** Loading states for different reports */
	loading: {
		financialOverview: boolean;
		spendingByCategory: boolean;
		monthlySpendingTrend: boolean;
		accountBalanceHistory: boolean;
		budgetPerformance: boolean;
		netWorthProgression: boolean;
	};
	/** Error state */
	error: FiscusApiError | null;
	/** Cache timestamps for reports */
	lastUpdated: {
		financialOverview: number | null;
		spendingByCategory: number | null;
		monthlySpendingTrend: number | null;
		accountBalanceHistory: number | null;
		budgetPerformance: number | null;
		netWorthProgression: number | null;
	};
}

interface ReportsActions {
	/** Load financial overview */
	loadFinancialOverview: (
		userId: string,
		startDate?: string,
		endDate?: string,
	) => Promise<void>;
	/** Load spending by category report */
	loadSpendingByCategory: (
		userId: string,
		startDate?: string,
		endDate?: string,
		limit?: number,
	) => Promise<void>;
	/** Load monthly spending trend */
	loadMonthlySpendingTrend: (userId: string, months?: number) => Promise<void>;
	/** Load account balance history */
	loadAccountBalanceHistory: (
		userId: string,
		accountId?: string,
		days?: number,
	) => Promise<void>;
	/** Load budget performance */
	loadBudgetPerformance: (
		userId: string,
		budgetPeriodId?: string,
	) => Promise<void>;
	/** Load net worth progression */
	loadNetWorthProgression: (userId: string, months?: number) => Promise<void>;
	/** Refresh all reports */
	refreshAllReports: (userId: string) => Promise<void>;
	/** Clear specific report data */
	clearReport: (reportType: keyof ReportsState["loading"]) => void;
	/** Clear error state */
	clearError: () => void;
	/** Reset store state */
	reset: () => void;
}

export type ReportsStore = ReportsState & ReportsActions;

const initialLoadingState = {
	financialOverview: false,
	spendingByCategory: false,
	monthlySpendingTrend: false,
	accountBalanceHistory: false,
	budgetPerformance: false,
	netWorthProgression: false,
};

const initialLastUpdatedState = {
	financialOverview: null,
	spendingByCategory: null,
	monthlySpendingTrend: null,
	accountBalanceHistory: null,
	budgetPerformance: null,
	netWorthProgression: null,
};

export const useReportsStore = create<ReportsStore>()((set, get) => ({
	// Initial state
	financialOverview: null,
	spendingByCategory: null,
	monthlySpendingTrend: null,
	accountBalanceHistory: null,
	budgetPerformance: null,
	netWorthProgression: null,
	loading: { ...initialLoadingState },
	error: null,
	lastUpdated: { ...initialLastUpdatedState },

	// Actions
	loadFinancialOverview: async (
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, financialOverview: true },
			error: null,
		}));

		try {
			const financialOverview = await apiClient.getFinancialOverview(
				userId,
				startDate,
				endDate,
			);

			set((state) => ({
				financialOverview,
				loading: { ...state.loading, financialOverview: false },
				lastUpdated: { ...state.lastUpdated, financialOverview: Date.now() },
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, financialOverview: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load financial overview",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	loadSpendingByCategory: async (
		userId: string,
		startDate?: string,
		endDate?: string,
		limit?: number,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, spendingByCategory: true },
			error: null,
		}));

		try {
			const spendingByCategory = await apiClient.getSpendingByCategory(
				userId,
				startDate,
				endDate,
				limit,
			);

			set((state) => ({
				spendingByCategory,
				loading: { ...state.loading, spendingByCategory: false },
				lastUpdated: { ...state.lastUpdated, spendingByCategory: Date.now() },
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, spendingByCategory: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load spending by category",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	loadMonthlySpendingTrend: async (
		userId: string,
		months?: number,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, monthlySpendingTrend: true },
			error: null,
		}));

		try {
			const monthlySpendingTrend = await apiClient.getMonthlySpendingTrend(
				userId,
				months,
			);

			set((state) => ({
				monthlySpendingTrend,
				loading: { ...state.loading, monthlySpendingTrend: false },
				lastUpdated: { ...state.lastUpdated, monthlySpendingTrend: Date.now() },
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, monthlySpendingTrend: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load monthly spending trend",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	loadAccountBalanceHistory: async (
		userId: string,
		accountId?: string,
		days?: number,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, accountBalanceHistory: true },
			error: null,
		}));

		try {
			const accountBalanceHistory = await apiClient.getAccountBalanceHistory(
				userId,
				accountId,
				days,
			);

			set((state) => ({
				accountBalanceHistory,
				loading: { ...state.loading, accountBalanceHistory: false },
				lastUpdated: {
					...state.lastUpdated,
					accountBalanceHistory: Date.now(),
				},
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, accountBalanceHistory: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load account balance history",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	loadBudgetPerformance: async (
		userId: string,
		budgetPeriodId?: string,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, budgetPerformance: true },
			error: null,
		}));

		try {
			const budgetPerformance = await apiClient.getBudgetPerformance(
				userId,
				budgetPeriodId,
			);

			set((state) => ({
				budgetPerformance,
				loading: { ...state.loading, budgetPerformance: false },
				lastUpdated: { ...state.lastUpdated, budgetPerformance: Date.now() },
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, budgetPerformance: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load budget performance",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	loadNetWorthProgression: async (
		userId: string,
		months?: number,
	): Promise<void> => {
		set((state) => ({
			loading: { ...state.loading, netWorthProgression: true },
			error: null,
		}));

		try {
			const netWorthProgression = await apiClient.getNetWorthProgression(
				userId,
				months,
			);

			set((state) => ({
				netWorthProgression,
				loading: { ...state.loading, netWorthProgression: false },
				lastUpdated: { ...state.lastUpdated, netWorthProgression: Date.now() },
				error: null,
			}));
		} catch (error) {
			set((state) => ({
				loading: { ...state.loading, netWorthProgression: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load net worth progression",
								"INTERNAL_ERROR",
							),
			}));
		}
	},

	refreshAllReports: async (userId: string): Promise<void> => {
		const actions = get();
		await Promise.all([
			actions.loadFinancialOverview(userId),
			actions.loadSpendingByCategory(userId),
			actions.loadMonthlySpendingTrend(userId),
			actions.loadAccountBalanceHistory(userId),
			actions.loadBudgetPerformance(userId),
			actions.loadNetWorthProgression(userId),
		]);
	},

	clearReport: (reportType: keyof ReportsState["loading"]) => {
		set((state) => ({
			[reportType]: null,
			lastUpdated: { ...state.lastUpdated, [reportType]: null },
		}));
	},

	clearError: () => {
		set({ error: null });
	},

	reset: () => {
		set({
			financialOverview: null,
			spendingByCategory: null,
			monthlySpendingTrend: null,
			accountBalanceHistory: null,
			budgetPerformance: null,
			netWorthProgression: null,
			loading: { ...initialLoadingState },
			error: null,
			lastUpdated: { ...initialLastUpdatedState },
		});
	},
}));

/**
 * Selector hooks for common report state
 */
export const useFinancialOverview = () => {
	const { financialOverview, loading, loadFinancialOverview } =
		useReportsStore();
	return {
		data: financialOverview,
		loading: loading.financialOverview,
		load: loadFinancialOverview,
	};
};

export const useSpendingByCategory = () => {
	const { spendingByCategory, loading, loadSpendingByCategory } =
		useReportsStore();
	return {
		data: spendingByCategory,
		loading: loading.spendingByCategory,
		load: loadSpendingByCategory,
	};
};

export const useMonthlySpendingTrend = () => {
	const { monthlySpendingTrend, loading, loadMonthlySpendingTrend } =
		useReportsStore();
	return {
		data: monthlySpendingTrend,
		loading: loading.monthlySpendingTrend,
		load: loadMonthlySpendingTrend,
	};
};

export const useAccountBalanceHistory = () => {
	const { accountBalanceHistory, loading, loadAccountBalanceHistory } =
		useReportsStore();
	return {
		data: accountBalanceHistory,
		loading: loading.accountBalanceHistory,
		load: loadAccountBalanceHistory,
	};
};

export const useBudgetPerformance = () => {
	const { budgetPerformance, loading, loadBudgetPerformance } =
		useReportsStore();
	return {
		data: budgetPerformance,
		loading: loading.budgetPerformance,
		load: loadBudgetPerformance,
	};
};

export const useNetWorthProgression = () => {
	const { netWorthProgression, loading, loadNetWorthProgression } =
		useReportsStore();
	return {
		data: netWorthProgression,
		loading: loading.netWorthProgression,
		load: loadNetWorthProgression,
	};
};

export const useReportsActions = () => {
	const { refreshAllReports, clearReport, clearError } = useReportsStore();
	return { refreshAllReports, clearReport, clearError };
};
