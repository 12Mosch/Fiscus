/**
 * Shared formatting utilities for consistent display across the application
 */

export interface CurrencyFormatOptions extends Intl.NumberFormatOptions {
	/** Whether to show the sign for positive numbers (default: false) */
	showPositiveSign?: boolean;
	/** Whether to handle negative values specially (default: false) */
	handleNegative?: boolean;
}

/**
 * Format currency with consistent settings across the application
 * Default: 2 decimal places for precise financial display
 */
export const formatCurrency = (
	amount: number,
	currency: string = "USD",
	options?: CurrencyFormatOptions,
): string => {
	const {
		showPositiveSign = false,
		handleNegative = false,
		...intlOptions
	} = options || {};

	const defaultOptions: Intl.NumberFormatOptions = {
		style: "currency",
		currency: currency,
		minimumFractionDigits: 2,
		maximumFractionDigits: 2,
		...intlOptions,
	};

	// Handle negative values specially if requested (like in AccountCard)
	if (handleNegative && amount < 0) {
		const absAmount = Math.abs(amount);
		const formatted = new Intl.NumberFormat("en-US", defaultOptions).format(
			absAmount,
		);
		return `-${formatted}`;
	}

	const formatted = new Intl.NumberFormat("en-US", defaultOptions).format(
		Math.abs(amount),
	);

	// Add positive sign if requested
	if (showPositiveSign && amount > 0) {
		return `+${formatted}`;
	}

	// Handle negative sign
	if (amount < 0) {
		return `-${formatted}`;
	}

	return formatted;
};

/**
 * Format currency without decimal places (for charts and summaries)
 */
export const formatCurrencyCompact = (
	amount: number,
	currency: string = "USD",
	options?: Omit<
		CurrencyFormatOptions,
		"minimumFractionDigits" | "maximumFractionDigits"
	>,
): string => {
	return formatCurrency(amount, currency, {
		...options,
		minimumFractionDigits: 0,
		maximumFractionDigits: 0,
	});
};

/**
 * Format currency with abbreviated notation for large amounts (K, M, B)
 */
export const formatCurrencyAbbreviated = (
	amount: number,
	currency: string = "USD",
): string => {
	const absAmount = Math.abs(amount);
	const sign = amount < 0 ? "-" : "";

	if (absAmount >= 1_000_000_000) {
		return `${sign}$${(absAmount / 1_000_000_000).toFixed(1)}B`;
	} else if (absAmount >= 1_000_000) {
		return `${sign}$${(absAmount / 1_000_000).toFixed(1)}M`;
	} else if (absAmount >= 1_000) {
		return `${sign}$${(absAmount / 1_000).toFixed(1)}K`;
	} else {
		return formatCurrencyCompact(amount, currency);
	}
};

/**
 * Format percentage with consistent decimal places
 */
export const formatPercentage = (
	value: number,
	decimals: number = 1,
): string => {
	return `${value.toFixed(decimals)}%`;
};

/**
 * Format date relative to now (e.g., "2h ago", "Yesterday")
 */
export const formatRelativeDate = (date: Date): string => {
	const now = new Date();
	const diffInMinutes = Math.floor(
		(now.getTime() - date.getTime()) / (1000 * 60),
	);

	if (diffInMinutes < 60) {
		return `${diffInMinutes}m ago`;
	} else if (diffInMinutes < 1440) {
		return `${Math.floor(diffInMinutes / 60)}h ago`;
	} else {
		const diffInDays = Math.floor(diffInMinutes / 1440);
		if (diffInDays === 1) {
			return "Yesterday";
		} else if (diffInDays < 7) {
			return `${diffInDays} days ago`;
		} else {
			return date.toLocaleDateString();
		}
	}
};

/**
 * Format transaction date (Today, Yesterday, or date)
 */
export const formatTransactionDate = (date: Date): string => {
	const now = new Date();
	const diffInDays = Math.floor(
		(now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24),
	);

	if (diffInDays === 0) {
		return "Today";
	} else if (diffInDays === 1) {
		return "Yesterday";
	} else if (diffInDays < 7) {
		return `${diffInDays} days ago`;
	} else {
		return date.toLocaleDateString();
	}
};
