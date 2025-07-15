/**
 * Custom hook for calculating month-over-month changes in dashboard metrics
 * Provides percentage changes for financial metrics to replace hardcoded values
 */

import { useMemo } from "react";
import {
	useAccountBalanceHistory,
	useMonthlySpendingTrend,
} from "./use-dashboard-data";

export interface DashboardChange {
	value: number;
	type: "increase" | "decrease";
	period: string;
}

export interface DashboardChanges {
	totalBalance: DashboardChange | null;
	monthlyIncome: DashboardChange | null;
	monthlyExpenses: DashboardChange | null;
	savingsRate: DashboardChange | null;
}

/**
 * Calculate percentage change between two values
 * @param current - Current value
 * @param previous - Previous value
 * @returns Percentage change (positive for increase, negative for decrease)
 */
function calculatePercentageChange(current: number, previous: number): number {
	if (previous === 0) {
		return current > 0 ? 100 : 0;
	}
	return ((current - previous) / Math.abs(previous)) * 100;
}

/**
 * Create a DashboardChange object from a percentage change value
 * @param percentageChange - The calculated percentage change
 * @param period - The period description (e.g., "last month")
 * @returns DashboardChange object or null if no valid change
 */
function createDashboardChange(
	percentageChange: number,
	period: string = "last month",
): DashboardChange | null {
	if (!Number.isFinite(percentageChange)) {
		return null;
	}

	return {
		value: Math.abs(percentageChange),
		type: percentageChange >= 0 ? "increase" : "decrease",
		period,
	};
}

/**
 * Hook to calculate month-over-month changes for dashboard metrics
 * Returns null for metrics when insufficient historical data is available
 * to ensure only real calculated changes are displayed
 * @returns Object containing change data for each metric (null when insufficient data)
 */
export function useDashboardChanges(): DashboardChanges {
	// Get historical data for calculations
	const { balanceHistory } = useAccountBalanceHistory(60); // Get 2 months of data
	const { monthlyTrend } = useMonthlySpendingTrend(3); // Get 3 months of trend data

	return useMemo(() => {
		const changes: DashboardChanges = {
			totalBalance: null,
			monthlyIncome: null,
			monthlyExpenses: null,
			savingsRate: null,
		};

		// Calculate total balance change from balance history
		if (balanceHistory && balanceHistory.length >= 2) {
			// Sort by date to ensure proper order
			const sortedHistory = [...balanceHistory].sort(
				(a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
			);

			const currentBalance =
				sortedHistory[sortedHistory.length - 1]?.value || 0;
			const previousBalance =
				sortedHistory[sortedHistory.length - 2]?.value || 0;

			const balanceChange = calculatePercentageChange(
				currentBalance,
				previousBalance,
			);
			changes.totalBalance = createDashboardChange(balanceChange);
		}

		// Calculate income and expense changes from monthly trend
		if (monthlyTrend && monthlyTrend.length >= 4) {
			// Group by month and calculate monthly totals
			const monthlyData = new Map<
				string,
				{ income: number; expenses: number }
			>();

			monthlyTrend.forEach((point) => {
				const month = point.date;
				if (!monthlyData.has(month)) {
					monthlyData.set(month, { income: 0, expenses: 0 });
				}

				const data = monthlyData.get(month);
				if (!data) return;
				if (point.label === "Income") {
					data.income += point.value;
				} else if (point.label === "Expenses") {
					data.expenses += point.value;
				}
			});

			const months = Array.from(monthlyData.keys()).sort();

			if (months.length >= 2) {
				const currentMonth = monthlyData.get(months[months.length - 1]);
				const previousMonth = monthlyData.get(months[months.length - 2]);

				if (currentMonth && previousMonth) {
					// Calculate income change
					const incomeChange = calculatePercentageChange(
						currentMonth.income,
						previousMonth.income,
					);
					changes.monthlyIncome = createDashboardChange(incomeChange);

					// Calculate expense change
					const expenseChange = calculatePercentageChange(
						currentMonth.expenses,
						previousMonth.expenses,
					);
					changes.monthlyExpenses = createDashboardChange(expenseChange);

					// Calculate savings rate change
					const currentSavingsRate =
						currentMonth.income > 0
							? ((currentMonth.income - currentMonth.expenses) /
									currentMonth.income) *
								100
							: 0;
					const previousSavingsRate =
						previousMonth.income > 0
							? ((previousMonth.income - previousMonth.expenses) /
									previousMonth.income) *
								100
							: 0;

					const savingsRateChange = calculatePercentageChange(
						currentSavingsRate,
						previousSavingsRate,
					);
					changes.savingsRate = createDashboardChange(savingsRateChange);
				}
			}
		}

		return changes;
	}, [balanceHistory, monthlyTrend]);
}
