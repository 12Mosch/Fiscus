/**
 * Tests for useBudgetSummary hook
 */

import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Budget } from "@/types/dashboard";
import { useBudgetSummary } from "../use-budget-summary";

describe("useBudgetSummary", () => {
	const mockBudgets: Budget[] = [
		{
			id: "budget-1",
			category: "Food",
			allocated: 1000,
			spent: 800, // 80% - on track
			currency: "USD",
			period: "monthly",
			startDate: new Date("2024-01-01"),
			endDate: new Date("2024-01-31"),
		},
		{
			id: "budget-2",
			category: "Transport",
			allocated: 500,
			spent: 475, // 95% - warning
			currency: "USD",
			period: "monthly",
			startDate: new Date("2024-01-01"),
			endDate: new Date("2024-01-31"),
		},
		{
			id: "budget-3",
			category: "Entertainment",
			allocated: 300,
			spent: 350, // 116.67% - over limit
			currency: "USD",
			period: "monthly",
			startDate: new Date("2024-01-01"),
			endDate: new Date("2024-01-31"),
		},
	];

	it("should categorize budgets correctly", () => {
		const { result } = renderHook(() => useBudgetSummary(mockBudgets));

		expect(result.current.onTrack).toBe(1);
		expect(result.current.warning).toBe(1);
		expect(result.current.overLimit).toBe(1);
	});

	it("should calculate total budgets correctly", () => {
		const { result } = renderHook(() => useBudgetSummary(mockBudgets));

		expect(result.current.totalBudgets).toBe(3);
	});

	it("should calculate total allocated and spent correctly", () => {
		const { result } = renderHook(() => useBudgetSummary(mockBudgets));

		expect(result.current.totalAllocated).toBe(1800); // 1000 + 500 + 300
		expect(result.current.totalSpent).toBe(1625); // 800 + 475 + 350
	});

	it("should calculate average utilization correctly", () => {
		const { result } = renderHook(() => useBudgetSummary(mockBudgets));

		// (80 + 95 + 116.67) / 3 = 97.22%
		expect(result.current.averageUtilization).toBeCloseTo(97.22, 1);
	});

	it("should handle empty budget array", () => {
		const { result } = renderHook(() => useBudgetSummary([]));

		expect(result.current.onTrack).toBe(0);
		expect(result.current.warning).toBe(0);
		expect(result.current.overLimit).toBe(0);
		expect(result.current.totalBudgets).toBe(0);
		expect(result.current.averageUtilization).toBe(0);
		expect(result.current.totalAllocated).toBe(0);
		expect(result.current.totalSpent).toBe(0);
	});

	it("should handle null/undefined budget array", () => {
		// biome-ignore lint/suspicious/noExplicitAny: Testing null handling behavior
		const { result } = renderHook(() => useBudgetSummary(null as any));

		expect(result.current.onTrack).toBe(0);
		expect(result.current.warning).toBe(0);
		expect(result.current.overLimit).toBe(0);
		expect(result.current.totalBudgets).toBe(0);
	});

	it("should handle zero allocated budget", () => {
		const budgetsWithZero: Budget[] = [
			{
				id: "budget-zero",
				category: "Test",
				allocated: 0,
				spent: 100,
				currency: "USD",
				period: "monthly",
				startDate: new Date("2024-01-01"),
				endDate: new Date("2024-01-31"),
			},
		];

		const { result } = renderHook(() => useBudgetSummary(budgetsWithZero));

		expect(result.current.onTrack).toBe(1); // 0% utilization is considered on track
	});

	it("should memoize results correctly", () => {
		const { result, rerender } = renderHook(() =>
			useBudgetSummary(mockBudgets),
		);

		const firstResult = result.current;
		rerender();
		const secondResult = result.current;

		expect(firstResult).toBe(secondResult);
	});

	it("should recalculate when budgets change", () => {
		const { result, rerender } = renderHook(
			({ budgets }) => useBudgetSummary(budgets),
			{ initialProps: { budgets: mockBudgets } },
		);

		const firstResult = result.current;

		const newBudgets: Budget[] = [
			...mockBudgets,
			{
				id: "budget-4",
				category: "Shopping",
				allocated: 200,
				spent: 50, // 25% - on track
				currency: "USD",
				period: "monthly" as const,
				startDate: new Date("2024-01-01"),
				endDate: new Date("2024-01-31"),
			},
		];

		rerender({ budgets: newBudgets });

		expect(result.current).not.toBe(firstResult);
		expect(result.current.totalBudgets).toBe(4);
		expect(result.current.onTrack).toBe(2);
	});
});
