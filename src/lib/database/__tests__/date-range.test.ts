/**
 * Tests for getDateRange function
 * Comprehensive tests for date calculation edge cases
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { dbUtils } from "../index";

describe("getDateRange", () => {
	// Mock Date to control test scenarios
	const mockDate = (dateString: string) => {
		const mockNow = new Date(dateString);
		vi.setSystemTime(mockNow);
		return mockNow;
	};

	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe("today", () => {
		it("should return start and end of today", () => {
			mockDate("2024-03-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("today");

			expect(result.start).toBe("2024-03-15");
			expect(result.end).toBe("2024-03-15");
		});

		it("should handle timezone correctly", () => {
			// Test with different times of day - use a time that won't cross date boundaries
			mockDate("2024-03-15T12:00:00.000Z");

			const result = dbUtils.getDateRange("today");

			// The result should be based on the local date, not UTC
			const testDate = new Date("2024-03-15T12:00:00.000Z");
			const expectedDate =
				testDate.getFullYear() +
				"-" +
				String(testDate.getMonth() + 1).padStart(2, "0") +
				"-" +
				String(testDate.getDate()).padStart(2, "0");

			expect(result.start).toBe(expectedDate);
			expect(result.end).toBe(expectedDate);
		});
	});

	describe("week", () => {
		it("should return exactly 7 days ago", () => {
			mockDate("2024-03-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("week");

			expect(result.start).toBe("2024-03-08");
			expect(result.end).toBe("2024-03-15");
		});

		it("should handle month boundaries", () => {
			mockDate("2024-03-05T14:30:00.000Z");

			const result = dbUtils.getDateRange("week");

			expect(result.start).toBe("2024-02-27");
			expect(result.end).toBe("2024-03-05");
		});

		it("should handle year boundaries", () => {
			mockDate("2024-01-05T14:30:00.000Z");

			const result = dbUtils.getDateRange("week");

			expect(result.start).toBe("2023-12-29");
			expect(result.end).toBe("2024-01-05");
		});
	});

	describe("month", () => {
		it("should return same day in previous month", () => {
			mockDate("2024-03-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("month");

			expect(result.start).toBe("2024-02-15");
			expect(result.end).toBe("2024-03-15");
		});

		it("should handle March 31 -> February edge case", () => {
			mockDate("2024-03-31T14:30:00.000Z");

			const result = dbUtils.getDateRange("month");

			// Should use Feb 29 (2024 is leap year)
			expect(result.start).toBe("2024-02-29");
			expect(result.end).toBe("2024-03-31");
		});

		it("should handle March 31 -> February in non-leap year", () => {
			mockDate("2023-03-31T14:30:00.000Z");

			const result = dbUtils.getDateRange("month");

			// Should use Feb 28 (2023 is not leap year)
			expect(result.start).toBe("2023-02-28");
			expect(result.end).toBe("2023-03-31");
		});

		it("should handle January -> December year boundary", () => {
			mockDate("2024-01-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("month");

			expect(result.start).toBe("2023-12-15");
			expect(result.end).toBe("2024-01-15");
		});

		it("should handle May 31 -> April 30 edge case", () => {
			mockDate("2024-05-31T14:30:00.000Z");

			const result = dbUtils.getDateRange("month");

			// Should use April 30 (April has only 30 days)
			expect(result.start).toBe("2024-04-30");
			expect(result.end).toBe("2024-05-31");
		});
	});

	describe("quarter", () => {
		it("should return same day 3 months ago", () => {
			mockDate("2024-06-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("quarter");

			expect(result.start).toBe("2024-03-15");
			expect(result.end).toBe("2024-06-15");
		});

		it("should handle year boundary", () => {
			mockDate("2024-02-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("quarter");

			expect(result.start).toBe("2023-11-15");
			expect(result.end).toBe("2024-02-15");
		});

		it("should handle May 31 -> February edge case", () => {
			mockDate("2024-05-31T14:30:00.000Z");

			const result = dbUtils.getDateRange("quarter");

			// Should use Feb 29 (2024 is leap year, February has only 29 days)
			expect(result.start).toBe("2024-02-29");
			expect(result.end).toBe("2024-05-31");
		});

		it("should handle August 31 -> May 31 edge case", () => {
			mockDate("2024-08-31T14:30:00.000Z");

			const result = dbUtils.getDateRange("quarter");

			// May has 31 days, so should be May 31
			expect(result.start).toBe("2024-05-31");
			expect(result.end).toBe("2024-08-31");
		});
	});

	describe("year", () => {
		it("should return same day one year ago", () => {
			mockDate("2024-03-15T14:30:00.000Z");

			const result = dbUtils.getDateRange("year");

			expect(result.start).toBe("2023-03-15");
			expect(result.end).toBe("2024-03-15");
		});

		it("should handle leap year Feb 29 -> Feb 28", () => {
			mockDate("2024-02-29T14:30:00.000Z");

			const result = dbUtils.getDateRange("year");

			// 2023 is not a leap year, so should use Feb 28
			expect(result.start).toBe("2023-02-28");
			expect(result.end).toBe("2024-02-29");
		});

		it("should handle leap year Feb 29 -> Feb 29", () => {
			mockDate("2020-02-29T14:30:00.000Z");

			const result = dbUtils.getDateRange("year");

			// 2019 is not a leap year, so should use Feb 28
			expect(result.start).toBe("2019-02-28");
			expect(result.end).toBe("2020-02-29");
		});

		it("should handle regular Feb 28", () => {
			mockDate("2023-02-28T14:30:00.000Z");

			const result = dbUtils.getDateRange("year");

			expect(result.start).toBe("2022-02-28");
			expect(result.end).toBe("2023-02-28");
		});
	});

	describe("edge cases and consistency", () => {
		it("should always return start <= end", () => {
			const testDates = [
				"2024-01-01T00:00:00.000Z",
				"2024-02-29T12:00:00.000Z",
				"2024-03-31T23:59:59.999Z",
				"2024-12-31T12:00:00.000Z",
			];

			const periods: Array<"today" | "week" | "month" | "quarter" | "year"> = [
				"today",
				"week",
				"month",
				"quarter",
				"year",
			];

			testDates.forEach((dateString) => {
				mockDate(dateString);
				periods.forEach((period) => {
					const result = dbUtils.getDateRange(period);
					expect(result.start <= result.end).toBe(true);
				});
			});
		});

		it("should return valid date strings", () => {
			mockDate("2024-03-15T14:30:00.000Z");

			const periods: Array<"today" | "week" | "month" | "quarter" | "year"> = [
				"today",
				"week",
				"month",
				"quarter",
				"year",
			];

			periods.forEach((period) => {
				const result = dbUtils.getDateRange(period);

				// Should be valid date strings in YYYY-MM-DD format
				expect(result.start).toMatch(/^\d{4}-\d{2}-\d{2}$/);
				expect(result.end).toMatch(/^\d{4}-\d{2}-\d{2}$/);

				// Should be parseable as valid dates
				expect(new Date(result.start).toString()).not.toBe("Invalid Date");
				expect(new Date(result.end).toString()).not.toBe("Invalid Date");
			});
		});
	});
});
