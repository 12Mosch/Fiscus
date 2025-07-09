/**
 * Custom hook for processing and filtering chart data
 * Extracts chart data manipulation logic for better testability and reusability
 */

import { useMemo } from "react";
import type { ChartDataPoint } from "@/types/dashboard";

export interface ChartDataFilters {
	limit?: number;
	labelFilter?: string;
	dateRange?: {
		start: Date;
		end: Date;
	};
}

/**
 * Hook to process and filter chart data
 * @param data - Array of chart data points
 * @param filters - Optional filters to apply to the data
 * @returns Processed and filtered chart data
 */
export function useChartData(
	data: ChartDataPoint[],
	filters?: ChartDataFilters,
): ChartDataPoint[] {
	return useMemo(() => {
		if (!data || data.length === 0) {
			return [];
		}

		let processedData = [...data];

		// Apply label filter
		if (filters?.labelFilter) {
			processedData = processedData.filter(
				(item) => item.label === filters.labelFilter,
			);
		}

		// Apply date range filter
		if (filters?.dateRange) {
			const { start, end } = filters.dateRange;
			processedData = processedData.filter((item) => {
				const itemDate = new Date(item.date);
				return itemDate >= start && itemDate <= end;
			});
		}

		// Apply limit
		if (filters?.limit && filters.limit > 0) {
			processedData = processedData.slice(0, filters.limit);
		}

		return processedData;
	}, [data, filters]);
}

/**
 * Hook to calculate chart data statistics
 * @param data - Array of chart data points
 * @returns Statistics about the chart data
 */
export function useChartStats(data: ChartDataPoint[]) {
	return useMemo(() => {
		if (!data || data.length === 0) {
			return {
				total: 0,
				average: 0,
				min: 0,
				max: 0,
				trend: "neutral" as "up" | "down" | "neutral",
			};
		}

		const values = data.map((item) => item.value);
		const total = values.reduce((sum, value) => sum + value, 0);
		const average = total / values.length;
		const min = Math.min(...values);
		const max = Math.max(...values);

		// Calculate trend (comparing first and last values)
		let trend: "up" | "down" | "neutral" = "neutral";
		if (values.length >= 2) {
			const firstValue = values[0];
			const lastValue = values[values.length - 1];
			const changePercent = ((lastValue - firstValue) / firstValue) * 100;

			if (changePercent > 5) {
				trend = "up";
			} else if (changePercent < -5) {
				trend = "down";
			}
		}

		return {
			total,
			average,
			min,
			max,
			trend,
		};
	}, [data]);
}
