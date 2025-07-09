/**
 * Custom hook for calculating budget summary statistics
 * Extracts budget calculation logic for better testability and reusability
 */

import { useMemo } from "react";
import type { Budget } from "@/types/dashboard";

export interface BudgetSummary {
	onTrack: number;
	warning: number;
	overLimit: number;
	totalBudgets: number;
	averageUtilization: number;
	totalAllocated: number;
	totalSpent: number;
}

/**
 * Hook to calculate budget summary statistics
 * @param budgets - Array of budget objects
 * @returns Budget summary with categorized counts and totals
 */
export function useBudgetSummary(budgets: Budget[]): BudgetSummary {
	return useMemo(() => {
		if (!budgets || budgets.length === 0) {
			return {
				onTrack: 0,
				warning: 0,
				overLimit: 0,
				totalBudgets: 0,
				averageUtilization: 0,
				totalAllocated: 0,
				totalSpent: 0,
			};
		}

		const summary = budgets.reduce(
			(acc, budget) => {
				const utilization =
					budget.allocated > 0 ? (budget.spent / budget.allocated) * 100 : 0;

				// Categorize budget status
				if (utilization > 100) {
					acc.overLimit++;
				} else if (utilization > 90) {
					acc.warning++;
				} else {
					acc.onTrack++;
				}

				// Accumulate totals
				acc.totalAllocated += budget.allocated;
				acc.totalSpent += budget.spent;
				acc.totalUtilization += utilization;

				return acc;
			},
			{
				onTrack: 0,
				warning: 0,
				overLimit: 0,
				totalAllocated: 0,
				totalSpent: 0,
				totalUtilization: 0,
			},
		);

		const averageUtilization = summary.totalUtilization / budgets.length;

		return {
			onTrack: summary.onTrack,
			warning: summary.warning,
			overLimit: summary.overLimit,
			totalBudgets: budgets.length,
			averageUtilization,
			totalAllocated: summary.totalAllocated,
			totalSpent: summary.totalSpent,
		};
	}, [budgets]);
}
