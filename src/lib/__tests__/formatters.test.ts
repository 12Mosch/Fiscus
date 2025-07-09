/**
 * Tests for formatting utilities
 */

import { describe, expect, it } from "vitest";
import {
	formatCurrency,
	formatCurrencyCompact,
	formatCurrencyAbbreviated,
	formatPercentage,
	formatRelativeDate,
	formatTransactionDate,
} from "../formatters";

describe("formatCurrency", () => {
	it("formats currency with default 2 decimal places", () => {
		expect(formatCurrency(1234.56, "USD")).toBe("$1,234.56");
		expect(formatCurrency(0, "USD")).toBe("$0.00");
		expect(formatCurrency(1000, "USD")).toBe("$1,000.00");
	});

	it("handles negative values correctly", () => {
		expect(formatCurrency(-1234.56, "USD")).toBe("-$1,234.56");
		expect(formatCurrency(-1234.56, "USD", { handleNegative: true })).toBe("-$1,234.56");
	});

	it("shows positive sign when requested", () => {
		expect(formatCurrency(1234.56, "USD", { showPositiveSign: true })).toBe("+$1,234.56");
		expect(formatCurrency(-1234.56, "USD", { showPositiveSign: true })).toBe("-$1,234.56");
	});

	it("respects custom fraction digits", () => {
		expect(formatCurrency(1234.567, "USD", { maximumFractionDigits: 3 })).toBe("$1,234.567");
		expect(formatCurrency(1234, "USD", { minimumFractionDigits: 0, maximumFractionDigits: 0 })).toBe("$1,234");
	});
});

describe("formatCurrencyCompact", () => {
	it("formats currency without decimal places", () => {
		expect(formatCurrencyCompact(1234.56, "USD")).toBe("$1,235");
		expect(formatCurrencyCompact(1000, "USD")).toBe("$1,000");
		expect(formatCurrencyCompact(0, "USD")).toBe("$0");
	});

	it("handles negative values", () => {
		expect(formatCurrencyCompact(-1234.56, "USD")).toBe("-$1,235");
	});
});

describe("formatCurrencyAbbreviated", () => {
	it("formats small amounts normally", () => {
		expect(formatCurrencyAbbreviated(999, "USD")).toBe("$999");
		expect(formatCurrencyAbbreviated(500, "USD")).toBe("$500");
	});

	it("formats thousands with K suffix", () => {
		expect(formatCurrencyAbbreviated(1000, "USD")).toBe("$1.0K");
		expect(formatCurrencyAbbreviated(1500, "USD")).toBe("$1.5K");
		expect(formatCurrencyAbbreviated(999999, "USD")).toBe("$1000.0K");
	});

	it("formats millions with M suffix", () => {
		expect(formatCurrencyAbbreviated(1000000, "USD")).toBe("$1.0M");
		expect(formatCurrencyAbbreviated(1500000, "USD")).toBe("$1.5M");
		expect(formatCurrencyAbbreviated(999999999, "USD")).toBe("$1000.0M");
	});

	it("formats billions with B suffix", () => {
		expect(formatCurrencyAbbreviated(1000000000, "USD")).toBe("$1.0B");
		expect(formatCurrencyAbbreviated(1500000000, "USD")).toBe("$1.5B");
	});

	it("handles negative values", () => {
		expect(formatCurrencyAbbreviated(-1000, "USD")).toBe("-$1.0K");
		expect(formatCurrencyAbbreviated(-1000000, "USD")).toBe("-$1.0M");
	});
});

describe("formatPercentage", () => {
	it("formats percentage with default 1 decimal place", () => {
		expect(formatPercentage(12.345)).toBe("12.3%");
		expect(formatPercentage(0)).toBe("0.0%");
		expect(formatPercentage(100)).toBe("100.0%");
	});

	it("respects custom decimal places", () => {
		expect(formatPercentage(12.345, 0)).toBe("12%");
		expect(formatPercentage(12.345, 2)).toBe("12.35%");
	});
});

describe("formatRelativeDate", () => {
	it("formats recent dates correctly", () => {
		const now = new Date();
		const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000);
		const twoHoursAgo = new Date(now.getTime() - 2 * 60 * 60 * 1000);

		expect(formatRelativeDate(fiveMinutesAgo)).toBe("5m ago");
		expect(formatRelativeDate(twoHoursAgo)).toBe("2h ago");
	});

	it("formats yesterday correctly", () => {
		const now = new Date();
		const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);

		expect(formatRelativeDate(yesterday)).toBe("Yesterday");
	});
});

describe("formatTransactionDate", () => {
	it("formats today as 'Today'", () => {
		const today = new Date();
		expect(formatTransactionDate(today)).toBe("Today");
	});

	it("formats yesterday as 'Yesterday'", () => {
		const now = new Date();
		const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);
		expect(formatTransactionDate(yesterday)).toBe("Yesterday");
	});

	it("formats recent days with 'X days ago'", () => {
		const now = new Date();
		const threeDaysAgo = new Date(now.getTime() - 3 * 24 * 60 * 60 * 1000);
		expect(formatTransactionDate(threeDaysAgo)).toBe("3 days ago");
	});
});
