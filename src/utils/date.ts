/**
 * Date formatting and manipulation utilities
 * Provides consistent date handling across the application
 */

/**
 * Format a date string or Date object for display
 * @param date The date to format (string or Date)
 * @param format The format type
 * @returns Formatted date string
 */
export function formatDate(
	date: string | Date,
	format: "short" | "medium" | "long" | "relative" = "medium",
): string {
	if (!date) {
		return "";
	}

	let dateObj: Date;

	// Handle string dates
	if (typeof date === "string") {
		dateObj = new Date(date);
	} else {
		dateObj = date;
	}

	// Validate date
	if (Number.isNaN(dateObj.getTime())) {
		return "Invalid Date";
	}

	const now = new Date();
	const diffInMs = now.getTime() - dateObj.getTime();
	const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

	switch (format) {
		case "short":
			return dateObj.toLocaleDateString("en-US", {
				month: "numeric",
				day: "numeric",
				year: "2-digit",
			});

		case "medium":
			return dateObj.toLocaleDateString("en-US", {
				month: "short",
				day: "numeric",
				year: "numeric",
			});

		case "long":
			return dateObj.toLocaleDateString("en-US", {
				weekday: "long",
				month: "long",
				day: "numeric",
				year: "numeric",
			});

		case "relative":
			if (diffInDays === 0) {
				return "Today";
			} else if (diffInDays === 1) {
				return "Yesterday";
			} else if (diffInDays === -1) {
				return "Tomorrow";
			} else if (diffInDays > 0 && diffInDays < 7) {
				return `${diffInDays} days ago`;
			} else if (diffInDays < 0 && diffInDays > -7) {
				return `In ${Math.abs(diffInDays)} days`;
			} else {
				return formatDate(dateObj, "medium");
			}

		default:
			return formatDate(dateObj, "medium");
	}
}

/**
 * Format a date for input fields (YYYY-MM-DD)
 * @param date The date to format
 * @returns Date string in YYYY-MM-DD format
 */
export function formatDateForInput(date: string | Date): string {
	if (!date) {
		return "";
	}

	let dateObj: Date;

	if (typeof date === "string") {
		dateObj = new Date(date);
	} else {
		dateObj = date;
	}

	if (Number.isNaN(dateObj.getTime())) {
		return "";
	}

	const year = dateObj.getFullYear();
	const month = String(dateObj.getMonth() + 1).padStart(2, "0");
	const day = String(dateObj.getDate()).padStart(2, "0");
	return `${year}-${month}-${day}`;
}

/**
 * Format a date with time
 * @param date The date to format
 * @param includeSeconds Whether to include seconds
 * @returns Formatted date and time string
 */
export function formatDateTime(
	date: string | Date,
	includeSeconds: boolean = false,
): string {
	if (!date) {
		return "";
	}

	let dateObj: Date;

	if (typeof date === "string") {
		dateObj = new Date(date);
	} else {
		dateObj = date;
	}

	if (Number.isNaN(dateObj.getTime())) {
		return "Invalid Date";
	}

	const dateStr = formatDate(dateObj, "medium");
	const timeStr = dateObj.toLocaleTimeString("en-US", {
		hour: "numeric",
		minute: "2-digit",
		second: includeSeconds ? "2-digit" : undefined,
		hour12: true,
	});

	return `${dateStr} at ${timeStr}`;
}

/**
 * Get the start and end of a date range
 * @param period The period type
 * @param date Optional reference date (defaults to today)
 * @returns Object with start and end dates
 */
export function getDateRange(
	period: "today" | "yesterday" | "week" | "month" | "quarter" | "year",
	date: Date = new Date(),
): { start: Date; end: Date } {
	const start = new Date(date);
	const end = new Date(date);

	switch (period) {
		case "today":
			start.setHours(0, 0, 0, 0);
			end.setHours(23, 59, 59, 999);
			break;

		case "yesterday":
			start.setDate(start.getDate() - 1);
			start.setHours(0, 0, 0, 0);
			end.setDate(end.getDate() - 1);
			end.setHours(23, 59, 59, 999);
			break;

		case "week": {
			const dayOfWeek = start.getDay();
			start.setDate(start.getDate() - dayOfWeek);
			start.setHours(0, 0, 0, 0);
			end.setDate(start.getDate() + 6);
			end.setHours(23, 59, 59, 999);
			break;
		}

		case "month":
			start.setDate(1);
			start.setHours(0, 0, 0, 0);
			end.setMonth(end.getMonth() + 1, 0);
			end.setHours(23, 59, 59, 999);
			break;

		case "quarter": {
			const quarter = Math.floor(start.getMonth() / 3);
			start.setMonth(quarter * 3, 1);
			start.setHours(0, 0, 0, 0);
			end.setMonth(quarter * 3 + 3, 0);
			end.setHours(23, 59, 59, 999);
			break;
		}

		case "year":
			start.setMonth(0, 1);
			start.setHours(0, 0, 0, 0);
			end.setMonth(11, 31);
			end.setHours(23, 59, 59, 999);
			break;
	}

	return { start, end };
}

/**
 * Check if a date is valid
 * @param date The date to validate
 * @returns True if valid, false otherwise
 */
export function isValidDate(date: string | Date): boolean {
	if (!date) {
		return false;
	}

	let dateObj: Date;

	if (typeof date === "string") {
		dateObj = new Date(date);
	} else {
		dateObj = date;
	}

	return !Number.isNaN(dateObj.getTime());
}

/**
 * Get the number of days between two dates
 * @param startDate The start date
 * @param endDate The end date
 * @returns Number of days between the dates
 */
export function getDaysBetween(
	startDate: string | Date,
	endDate: string | Date,
): number {
	const start = typeof startDate === "string" ? new Date(startDate) : startDate;
	const end = typeof endDate === "string" ? new Date(endDate) : endDate;

	if (!isValidDate(start) || !isValidDate(end)) {
		return 0;
	}

	const diffInMs = end.getTime() - start.getTime();
	return Math.floor(diffInMs / (1000 * 60 * 60 * 24));
}

/**
 * Add days to a date
 * @param date The base date
 * @param days Number of days to add (can be negative)
 * @returns New date with days added
 */
export function addDays(date: string | Date, days: number): Date {
	const dateObj = typeof date === "string" ? new Date(date) : new Date(date);
	dateObj.setDate(dateObj.getDate() + days);
	return dateObj;
}

/**
 * Format a date range for display
 * @param startDate The start date
 * @param endDate The end date
 * @returns Formatted date range string
 */
export function formatDateRange(
	startDate: string | Date,
	endDate: string | Date,
): string {
	if (!startDate || !endDate) {
		return "";
	}

	const start = formatDate(startDate, "medium");
	const end = formatDate(endDate, "medium");

	if (start === end) {
		return start;
	}

	return `${start} - ${end}`;
}
