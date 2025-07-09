/**
 * Custom hook for calculating dashboard financial metrics
 * Extracts calculation logic from components for better testability and reusability
 */

import { useMemo } from "react";
import type { DashboardStats } from "@/types/dashboard";

export interface DashboardMetrics {
	netWorth: number;
	monthlyNet: number;
	expenseRatio: number;
	savingsAmount: number;
	expenseRatioFormatted: string;
}

/**
 * Hook to calculate key financial metrics from dashboard stats
 * @param stats - Dashboard statistics object
 * @returns Calculated financial metrics
 */
export function useDashboardMetrics(stats: DashboardStats): DashboardMetrics {
	return useMemo(() => {
		const netWorth = stats.totalBalance;
		const monthlyNet = stats.monthlyIncome - stats.monthlyExpenses;
		const expenseRatio =
			stats.monthlyIncome > 0
				? (stats.monthlyExpenses / stats.monthlyIncome) * 100
				: 0;
		const savingsAmount = stats.monthlyIncome * (stats.savingsRate / 100);
		const expenseRatioFormatted = `${expenseRatio.toFixed(1)}%`;

		return {
			netWorth,
			monthlyNet,
			expenseRatio,
			savingsAmount,
			expenseRatioFormatted,
		};
	}, [stats]);
}
