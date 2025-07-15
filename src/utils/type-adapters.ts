/**
 * Type adapters to convert between API types and Dashboard component types
 * This handles the field name differences between the backend API and frontend components
 */

import type {
	Account as ApiAccount,
	Budget as ApiBudget,
	Transaction as ApiTransaction,
} from "@/types/api";
import type {
	Account as DashboardAccount,
	Budget as DashboardBudget,
	Transaction as DashboardTransaction,
} from "@/types/dashboard";

/**
 * Convert API Account to Dashboard Account
 */
export function adaptApiAccountToDashboard(
	apiAccount: ApiAccount,
): DashboardAccount {
	// Map account_type_id to readable type
	const getAccountType = (accountTypeId: string): DashboardAccount["type"] => {
		// This is a simplified mapping - in a real app you'd fetch account types
		// and map based on the actual account type data
		switch (accountTypeId.toLowerCase()) {
			case "checking":
			case "chequing":
				return "checking";
			case "savings":
				return "savings";
			case "credit":
			case "credit_card":
				return "credit";
			case "investment":
			case "brokerage":
				return "investment";
			default:
				return "checking"; // Default fallback
		}
	};

	return {
		id: apiAccount.id,
		name: apiAccount.name,
		type: getAccountType(apiAccount.account_type_id),
		balance: apiAccount.balance,
		currency: apiAccount.currency,
		lastUpdated: new Date(apiAccount.updated_at),
		accountNumber: apiAccount.account_number,
		// Note: creditLimit is not in API Account type, would need to be added
	};
}

/**
 * Convert API Transaction to Dashboard Transaction
 */
export function adaptApiTransactionToDashboard(
	apiTransaction: ApiTransaction,
): DashboardTransaction {
	return {
		id: apiTransaction.id,
		accountId: apiTransaction.account_id,
		amount: apiTransaction.amount,
		description: apiTransaction.description,
		category: apiTransaction.category_id || "Uncategorized",
		date: new Date(apiTransaction.transaction_date),
		type: apiTransaction.transaction_type as DashboardTransaction["type"],
		status: apiTransaction.status as DashboardTransaction["status"],
		merchant: apiTransaction.payee,
		tags: apiTransaction.tags,
	};
}

/**
 * Convert API Budget to Dashboard Budget
 * Note: This is a simplified conversion as the API Budget structure is quite different
 */
export function adaptApiBudgetToDashboard(
	apiBudget: ApiBudget,
): DashboardBudget {
	// Generate a readable category name from category_id
	const getCategoryName = (categoryId: string): string => {
		// This is a simplified mapping - in a real app you'd fetch category names
		const categoryMap: Record<string, string> = {
			food: "Food & Dining",
			transport: "Transportation",
			entertainment: "Entertainment",
			shopping: "Shopping",
			utilities: "Utilities",
			healthcare: "Healthcare",
		};

		return (
			categoryMap[categoryId.toLowerCase()] ||
			categoryId.charAt(0).toUpperCase() + categoryId.slice(1)
		);
	};

	// Generate a color based on category
	const getCategoryColor = (categoryId: string): string => {
		const colors = [
			"#ef4444",
			"#3b82f6",
			"#10b981",
			"#f59e0b",
			"#8b5cf6",
			"#06b6d4",
			"#84cc16",
		];
		const hash = categoryId.split("").reduce((a, b) => {
			a = (a << 5) - a + b.charCodeAt(0);
			return a & a;
		}, 0);
		return colors[Math.abs(hash) % colors.length];
	};

	return {
		id: apiBudget.id,
		category: getCategoryName(apiBudget.category_id),
		allocated: apiBudget.allocated_amount,
		spent: apiBudget.spent_amount,
		currency: "USD", // Default currency, would need to be fetched from user preferences
		period: "monthly" as const, // Default period, would need to be determined from budget period
		startDate: new Date(), // Would need to be fetched from budget period
		endDate: new Date(), // Would need to be fetched from budget period
		color: getCategoryColor(apiBudget.category_id),
	};
}

/**
 * Convert array of API Accounts to Dashboard Accounts
 */
export function adaptApiAccountsToDashboard(
	apiAccounts: ApiAccount[],
): DashboardAccount[] {
	return apiAccounts.map(adaptApiAccountToDashboard);
}

/**
 * Convert array of API Transactions to Dashboard Transactions
 */
export function adaptApiTransactionsToDashboard(
	apiTransactions: ApiTransaction[],
): DashboardTransaction[] {
	return apiTransactions.map(adaptApiTransactionToDashboard);
}

/**
 * Convert array of API Budgets to Dashboard Budgets
 */
export function adaptApiBudgetsToDashboard(
	apiBudgets: ApiBudget[],
): DashboardBudget[] {
	return apiBudgets.map(adaptApiBudgetToDashboard);
}
