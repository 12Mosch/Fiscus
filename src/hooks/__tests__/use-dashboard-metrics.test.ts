/**
 * Tests for useDashboardMetrics hook
 */

import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { DashboardStats } from "@/types/dashboard";
import { useDashboardMetrics } from "../use-dashboard-metrics";

describe("useDashboardMetrics", () => {
	const mockStats: DashboardStats = {
		totalBalance: 25000,
		monthlyIncome: 5000,
		monthlyExpenses: 3500,
		savingsRate: 30,
		currency: "USD",
	};

	it("should calculate net worth correctly", () => {
		const { result } = renderHook(() => useDashboardMetrics(mockStats));

		expect(result.current.netWorth).toBe(25000);
	});

	it("should calculate monthly net correctly", () => {
		const { result } = renderHook(() => useDashboardMetrics(mockStats));

		expect(result.current.monthlyNet).toBe(1500); // 5000 - 3500
	});

	it("should calculate expense ratio correctly", () => {
		const { result } = renderHook(() => useDashboardMetrics(mockStats));

		expect(result.current.expenseRatio).toBe(70); // (3500 / 5000) * 100
		expect(result.current.expenseRatioFormatted).toBe("70.0%");
	});

	it("should calculate savings amount correctly", () => {
		const { result } = renderHook(() => useDashboardMetrics(mockStats));

		expect(result.current.savingsAmount).toBe(1500); // 5000 * (30 / 100)
	});

	it("should handle zero income gracefully", () => {
		const zeroIncomeStats: DashboardStats = {
			...mockStats,
			monthlyIncome: 0,
		};

		const { result } = renderHook(() => useDashboardMetrics(zeroIncomeStats));

		expect(result.current.expenseRatio).toBe(0);
		expect(result.current.expenseRatioFormatted).toBe("0.0%");
		expect(result.current.monthlyNet).toBe(-3500);
	});

	it("should handle negative monthly net", () => {
		const highExpenseStats: DashboardStats = {
			...mockStats,
			monthlyExpenses: 6000,
		};

		const { result } = renderHook(() => useDashboardMetrics(highExpenseStats));

		expect(result.current.monthlyNet).toBe(-1000); // 5000 - 6000
	});

	it("should memoize results correctly", () => {
		const { result, rerender } = renderHook(() =>
			useDashboardMetrics(mockStats),
		);

		const firstResult = result.current;
		rerender();
		const secondResult = result.current;

		expect(firstResult).toBe(secondResult);
	});

	it("should recalculate when stats change", () => {
		const { result, rerender } = renderHook(
			({ stats }) => useDashboardMetrics(stats),
			{ initialProps: { stats: mockStats } },
		);

		const firstResult = result.current;

		const newStats: DashboardStats = {
			...mockStats,
			monthlyIncome: 6000,
		};

		rerender({ stats: newStats });

		expect(result.current).not.toBe(firstResult);
		expect(result.current.monthlyNet).toBe(2500); // 6000 - 3500
	});
});
