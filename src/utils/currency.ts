/**
 * Currency formatting utilities
 * Provides consistent currency formatting across the application
 */

/**
 * Format a number as currency
 * @param amount The amount to format
 * @param currency The currency code (default: USD)
 * @param locale The locale for formatting (default: en-US)
 * @returns Formatted currency string
 */
export function formatCurrency(
	amount: number,
	currency: string = "USD",
	locale: string = "en-US",
): string {
	try {
		return new Intl.NumberFormat(locale, {
			style: "currency",
			currency,
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(amount);
	} catch (_error) {
		// Fallback for invalid currency codes or locales
		return `${currency} ${amount.toFixed(2)}`;
	}
}

/**
 * Format a number as currency without the currency symbol
 * @param amount The amount to format
 * @param locale The locale for formatting (default: en-US)
 * @returns Formatted number string
 */
export function formatAmount(amount: number, locale: string = "en-US"): string {
	try {
		return new Intl.NumberFormat(locale, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(amount);
	} catch (_error) {
		// Fallback for invalid locales
		return amount.toFixed(2);
	}
}

/**
 * Parse a currency string to a number
 * @param currencyString The currency string to parse
 * @returns Parsed number or null if invalid
 */
export function parseCurrency(currencyString: string): number | null {
	if (!currencyString || typeof currencyString !== "string") {
		return null;
	}

	// Remove currency symbols, spaces, commas, and parentheses
	const cleanString = currencyString
		.replace(/[\p{Sc}]/gu, "") // All Unicode currency symbols
		.replace(/[,\s]/g, "") // Commas and spaces
		.replace(/[()]/g, "") // Parentheses for negative amounts
		.trim();

	// Handle negative amounts in parentheses (accounting format)
	const isNegative =
		currencyString.includes("(") && currencyString.includes(")");
	const parsed = Number.parseFloat(cleanString);
	const result = Number.isNaN(parsed) ? null : parsed;
	return result !== null && isNegative ? -result : result;
}

/**
 * Format currency with color coding for positive/negative amounts
 * @param amount The amount to format
 * @param currency The currency code (default: USD)
 * @param locale The locale for formatting (default: en-US)
 * @returns Object with formatted string and color class
 */
export function formatCurrencyWithColor(
	amount: number,
	currency: string = "USD",
	locale: string = "en-US",
	positiveClass: string = "text-green-600",
	negativeClass: string = "text-red-600",
): { formatted: string; colorClass: string } {
	const formatted = formatCurrency(amount, currency, locale);
	const colorClass = amount >= 0 ? positiveClass : negativeClass;

	return { formatted, colorClass };
}

/**
 * Format currency for display in tables or compact spaces
 * @param amount The amount to format
 * @param currency The currency code (default: USD)
 * @param showSign Whether to show + for positive amounts
 * @returns Compact formatted currency string
 */
export function formatCurrencyCompact(
	amount: number,
	currency: string = "USD",
	showSign: boolean = false,
): string {
	const sign = showSign && amount > 0 ? "+" : "";
	const formatted = formatCurrency(Math.abs(amount), currency);

	return amount < 0 ? `-${formatted}` : `${sign}${formatted}`;
}

/**
 * Get currency symbol for a given currency code
 * @param currency The currency code
 * @param locale The locale for formatting (default: en-US)
 * @returns Currency symbol
 */
export function getCurrencySymbol(
	currency: string = "USD",
	locale: string = "en-US",
): string {
	try {
		return (
			new Intl.NumberFormat(locale, {
				style: "currency",
				currency,
			})
				.formatToParts(0)
				.find((part) => part.type === "currency")?.value || currency
		);
	} catch (_error) {
		// Fallback for invalid currency codes
		const symbols: Record<string, string> = {
			USD: "$",
			EUR: "€",
			GBP: "£",
			JPY: "¥",
			CAD: "C$",
			AUD: "A$",
		};
		return symbols[currency] || currency;
	}
}

/**
 * Validate if a string is a valid currency code
 * @param currency The currency code to validate
 * @returns True if valid, false otherwise
 */
export function isValidCurrency(currency: string): boolean {
	try {
		new Intl.NumberFormat("en-US", {
			style: "currency",
			currency,
		}).format(0);
		return true;
	} catch (_error) {
		return false;
	}
}

/**
 * Convert amount between currencies using provided exchange rate
 * @param warning This function does not fetch live exchange rates
 * @param amount The amount to convert
 * @param fromCurrency Source currency
 * @param toCurrency Target currency
 * @param exchangeRate The exchange rate (from -> to)
 * @returns Converted amount
 */
// TODO: Fetch live exchange rates from an API
export function convertCurrency(
	amount: number,
	fromCurrency: string,
	toCurrency: string,
	exchangeRate: number,
): number {
	if (fromCurrency === toCurrency) {
		return amount;
	}
	if (exchangeRate <= 0) {
		throw new Error("Exchange rate must be positive");
	}
	return amount * exchangeRate;
}
