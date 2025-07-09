/**
 * Tests for useChartData and useChartStats hooks
 */

import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { ChartDataPoint } from "@/types/dashboard";
import { useChartData, useChartStats } from "../use-chart-data";

describe("useChartData", () => {
	const mockData: ChartDataPoint[] = [
		{ date: "2024-01-01", value: 1000, label: "Income" },
		{ date: "2024-01-02", value: 1200, label: "Income" },
		{ date: "2024-01-03", value: 800, label: "Expense" },
		{ date: "2024-01-04", value: 1100, label: "Income" },
		{ date: "2024-01-05", value: 900, label: "Expense" },
		{ date: "2024-01-06", value: 1300, label: "Income" },
		{ date: "2024-01-07", value: 700, label: "Expense" },
	];

	it("should return original data when no filters applied", () => {
		const { result } = renderHook(() => useChartData(mockData));

		expect(result.current).toEqual(mockData);
	});

	it("should filter by label correctly", () => {
		const { result } = renderHook(() =>
			useChartData(mockData, { labelFilter: "Income" }),
		);

		expect(result.current).toHaveLength(4);
		expect(result.current.every((item) => item.label === "Income")).toBe(true);
	});

	it("should apply limit correctly", () => {
		const { result } = renderHook(() => useChartData(mockData, { limit: 3 }));

		expect(result.current).toHaveLength(3);
		expect(result.current).toEqual(mockData.slice(0, 3));
	});

	it("should combine label filter and limit", () => {
		const { result } = renderHook(() =>
			useChartData(mockData, { labelFilter: "Income", limit: 2 }),
		);

		expect(result.current).toHaveLength(2);
		expect(result.current.every((item) => item.label === "Income")).toBe(true);
	});

	it("should filter by date range correctly", () => {
		const { result } = renderHook(() =>
			useChartData(mockData, {
				dateRange: {
					start: new Date("2024-01-02"),
					end: new Date("2024-01-04"),
				},
			}),
		);

		expect(result.current).toHaveLength(3);
		expect(result.current.map((item) => item.date)).toEqual([
			"2024-01-02",
			"2024-01-03",
			"2024-01-04",
		]);
	});

	it("should handle empty data array", () => {
		const { result } = renderHook(() => useChartData([]));

		expect(result.current).toEqual([]);
	});

	it("should handle null/undefined data", () => {
		const { result } = renderHook(() => useChartData(null as any));

		expect(result.current).toEqual([]);
	});

	it("should memoize results correctly", () => {
		const filters = { limit: 3 };
		const { result, rerender } = renderHook(() =>
			useChartData(mockData, filters),
		);

		const firstResult = result.current;
		rerender();
		const secondResult = result.current;

		expect(firstResult).toBe(secondResult);
	});
});

describe("useChartStats", () => {
	const mockData: ChartDataPoint[] = [
		{ date: "2024-01-01", value: 1000 },
		{ date: "2024-01-02", value: 1200 },
		{ date: "2024-01-03", value: 800 },
		{ date: "2024-01-04", value: 1400 },
	];

	it("should calculate statistics correctly", () => {
		const { result } = renderHook(() => useChartStats(mockData));

		expect(result.current.total).toBe(4400);
		expect(result.current.average).toBe(1100);
		expect(result.current.min).toBe(800);
		expect(result.current.max).toBe(1400);
	});

	it("should calculate upward trend correctly", () => {
		const trendUpData: ChartDataPoint[] = [
			{ date: "2024-01-01", value: 1000 },
			{ date: "2024-01-02", value: 1200 },
		];

		const { result } = renderHook(() => useChartStats(trendUpData));

		expect(result.current.trend).toBe("up"); // 20% increase
	});

	it("should calculate downward trend correctly", () => {
		const trendDownData: ChartDataPoint[] = [
			{ date: "2024-01-01", value: 1000 },
			{ date: "2024-01-02", value: 800 },
		];

		const { result } = renderHook(() => useChartStats(trendDownData));

		expect(result.current.trend).toBe("down"); // 20% decrease
	});

	it("should calculate neutral trend correctly", () => {
		const neutralData: ChartDataPoint[] = [
			{ date: "2024-01-01", value: 1000 },
			{ date: "2024-01-02", value: 1030 },
		];

		const { result } = renderHook(() => useChartStats(neutralData));

		expect(result.current.trend).toBe("neutral"); // 3% change
	});

	it("should handle empty data array", () => {
		const { result } = renderHook(() => useChartStats([]));

		expect(result.current.total).toBe(0);
		expect(result.current.average).toBe(0);
		expect(result.current.min).toBe(0);
		expect(result.current.max).toBe(0);
		expect(result.current.trend).toBe("neutral");
	});

	it("should handle single data point", () => {
		const singleData: ChartDataPoint[] = [{ date: "2024-01-01", value: 1000 }];

		const { result } = renderHook(() => useChartStats(singleData));

		expect(result.current.total).toBe(1000);
		expect(result.current.average).toBe(1000);
		expect(result.current.min).toBe(1000);
		expect(result.current.max).toBe(1000);
		expect(result.current.trend).toBe("neutral");
	});
});
