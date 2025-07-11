import { describe, expect, it } from "vitest";
import { formatDateForDb, formatDateTimeForDb } from "../connection";

describe("Date Formatting Functions", () => {
	describe("formatDateForDb", () => {
		it("should format Date object correctly", () => {
			const date = new Date("2024-01-15T10:30:00Z");
			const result = formatDateForDb(date);
			expect(result).toBe("2024-01-15");
		});

		it("should format valid date string correctly", () => {
			const dateString = "2024-01-15T10:30:00Z";
			const result = formatDateForDb(dateString);
			expect(result).toBe("2024-01-15");
		});

		it("should not create unnecessary Date object when input is already Date", () => {
			const originalDate = new Date("2024-01-15T10:30:00Z");
			const result = formatDateForDb(originalDate);
			expect(result).toBe("2024-01-15");
		});

		it("should throw error for invalid date string", () => {
			expect(() => formatDateForDb("invalid-date")).toThrow(
				"Invalid date: invalid-date",
			);
		});

		it("should throw error for empty string", () => {
			expect(() => formatDateForDb("")).toThrow("Date string cannot be empty");
		});

		it("should throw error for whitespace-only string", () => {
			expect(() => formatDateForDb("   ")).toThrow(
				"Date string cannot be empty",
			);
		});

		it("should throw error for invalid Date object", () => {
			const invalidDate = new Date("invalid");
			expect(() => formatDateForDb(invalidDate)).toThrow("Invalid date:");
		});

		it("should throw error for non-string, non-Date input", () => {
			expect(() => formatDateForDb(123 as unknown as string)).toThrow(
				"Date must be a Date object or a valid date string",
			);
		});
	});

	describe("formatDateTimeForDb", () => {
		it("should format Date object correctly", () => {
			const date = new Date("2024-01-15T10:30:00.000Z");
			const result = formatDateTimeForDb(date);
			expect(result).toBe("2024-01-15T10:30:00.000Z");
		});

		it("should format valid date string correctly", () => {
			const dateString = "2024-01-15T10:30:00Z";
			const result = formatDateTimeForDb(dateString);
			expect(result).toBe("2024-01-15T10:30:00.000Z");
		});

		it("should not create unnecessary Date object when input is already Date", () => {
			const originalDate = new Date("2024-01-15T10:30:00.000Z");
			const result = formatDateTimeForDb(originalDate);
			expect(result).toBe("2024-01-15T10:30:00.000Z");
		});

		it("should throw error for invalid date string", () => {
			expect(() => formatDateTimeForDb("invalid-date")).toThrow(
				"Invalid date: invalid-date",
			);
		});

		it("should throw error for empty string", () => {
			expect(() => formatDateTimeForDb("")).toThrow(
				"Date string cannot be empty",
			);
		});

		it("should throw error for whitespace-only string", () => {
			expect(() => formatDateTimeForDb("   ")).toThrow(
				"Date string cannot be empty",
			);
		});

		it("should throw error for invalid Date object", () => {
			const invalidDate = new Date("invalid");
			expect(() => formatDateTimeForDb(invalidDate)).toThrow("Invalid date:");
		});

		it("should throw error for non-string, non-Date input", () => {
			expect(() => formatDateTimeForDb(null as unknown as string)).toThrow(
				"Date must be a Date object or a valid date string",
			);
		});
	});
});
