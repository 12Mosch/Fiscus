/**
 * Tests for useDashboardChanges hook
 */

import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDashboardChanges } from "../use-dashboard-changes";

// Mock the dashboard data hooks
vi.mock("../use-dashboard-data", () => ({
	useAccountBalanceHistory: vi.fn(),
	useMonthlySpendingTrend: vi.fn(),
}));

import {
	useAccountBalanceHistory,
	useMonthlySpendingTrend,
} from "../use-dashboard-data";

const mockUseAccountBalanceHistory = useAccountBalanceHistory as ReturnType<
	typeof vi.fn
>;
const mockUseMonthlySpendingTrend = useMonthlySpendingTrend as ReturnType<
	typeof vi.fn
>;

describe("useDashboardChanges", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("should return null changes when no historical data is available", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		// All changes should be null when no historical data is available
		expect(result.current.totalBalance).toBeNull();
		expect(result.current.monthlyIncome).toBeNull();
		expect(result.current.monthlyExpenses).toBeNull();
		expect(result.current.savingsRate).toBeNull();
	});

	it("should calculate balance change correctly", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [
				{ date: "2024-01-01", value: 20000, label: "Balance: $20,000" },
				{ date: "2024-02-01", value: 25000, label: "Balance: $25,000" },
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		expect(result.current.totalBalance).toEqual({
			value: 25, // 25% increase from 20000 to 25000
			type: "increase",
			period: "last month",
		});
	});

	it("should calculate income and expense changes correctly", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [
				{ date: "2024-01", value: 4000, label: "Income" },
				{ date: "2024-01", value: 3000, label: "Expenses" },
				{ date: "2024-02", value: 5000, label: "Income" },
				{ date: "2024-02", value: 3500, label: "Expenses" },
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		expect(result.current.monthlyIncome).toEqual({
			value: 25, // 25% increase from 4000 to 5000
			type: "increase",
			period: "last month",
		});

		expect(result.current.monthlyExpenses).toEqual({
			value: 16.666666666666664, // ~16.67% increase from 3000 to 3500
			type: "increase",
			period: "last month",
		});
	});

	it("should handle decrease changes correctly", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [
				{ date: "2024-01-01", value: 30000, label: "Balance: $30,000" },
				{ date: "2024-02-01", value: 25000, label: "Balance: $25,000" },
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		expect(result.current.totalBalance).toEqual({
			value: 16.666666666666664, // ~16.67% decrease from 30000 to 25000
			type: "decrease",
			period: "last month",
		});
	});

	it("should handle zero previous values correctly", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [
				{ date: "2024-01-01", value: 0, label: "Balance: $0" },
				{ date: "2024-02-01", value: 25000, label: "Balance: $25,000" },
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		expect(result.current.totalBalance).toEqual({
			value: 100, // 100% increase when starting from 0
			type: "increase",
			period: "last month",
		});
	});

	it("should calculate savings rate change correctly", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [
				{ date: "2024-01", value: 4000, label: "Income" },
				{ date: "2024-01", value: 3200, label: "Expenses" }, // 20% savings rate
				{ date: "2024-02", value: 5000, label: "Income" },
				{ date: "2024-02", value: 3500, label: "Expenses" }, // 30% savings rate
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		// Savings rate went from 20% to 30%, which is a 50% increase
		expect(result.current.savingsRate).toEqual({
			value: 50,
			type: "increase",
			period: "last month",
		});
	});

	it("should return null when insufficient data is available", () => {
		mockUseAccountBalanceHistory.mockReturnValue({
			balanceHistory: [
				{ date: "2024-02-01", value: 25000, label: "Balance: $25,000" },
			],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		mockUseMonthlySpendingTrend.mockReturnValue({
			monthlyTrend: [{ date: "2024-02", value: 5000, label: "Income" }],
			loading: false,
			error: null,
			refetch: vi.fn(),
		});

		const { result } = renderHook(() => useDashboardChanges());

		// Should return null for metrics without enough historical data
		expect(result.current.totalBalance).toBeNull(); // Only one balance data point
		expect(result.current.monthlyIncome).toBeNull(); // Only one month of data
		expect(result.current.monthlyExpenses).toBeNull(); // Only one month of data
		expect(result.current.savingsRate).toBeNull(); // Only one month of data
	});
});
