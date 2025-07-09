/**
 * Mock data for the Finance Dashboard
 */

import type {
	Account,
	Budget,
	ChartDataPoint,
	DashboardStats,
	FinancialGoal,
	NavigationItem,
	NotificationItem,
	SpendingCategory,
	Transaction,
} from "../types/dashboard";

// Mock Accounts
export const mockAccounts: Account[] = [
	{
		id: "acc-1",
		name: "Main Checking",
		type: "checking",
		balance: 5420.5,
		currency: "USD",
		lastUpdated: new Date("2024-01-15T10:30:00Z"),
		accountNumber: "****1234",
	},
	{
		id: "acc-2",
		name: "High Yield Savings",
		type: "savings",
		balance: 15750.0,
		currency: "USD",
		lastUpdated: new Date("2024-01-15T09:15:00Z"),
		accountNumber: "****5678",
	},
	{
		id: "acc-3",
		name: "Credit Card",
		type: "credit",
		balance: -1250.75,
		currency: "USD",
		lastUpdated: new Date("2024-01-15T11:45:00Z"),
		accountNumber: "****9012",
	},
	{
		id: "acc-4",
		name: "Investment Portfolio",
		type: "investment",
		balance: 42350.25,
		currency: "USD",
		lastUpdated: new Date("2024-01-15T16:00:00Z"),
		accountNumber: "****3456",
	},
];

// Mock Transactions
export const mockTransactions: Transaction[] = [
	{
		id: "txn-1",
		accountId: "acc-1",
		amount: -85.5,
		description: "Grocery Store Purchase",
		category: "Food & Dining",
		date: new Date("2024-01-15T14:30:00Z"),
		type: "expense",
		status: "completed",
		merchant: "Whole Foods Market",
		tags: ["groceries", "food"],
	},
	{
		id: "txn-2",
		accountId: "acc-1",
		amount: 3200.0,
		description: "Salary Deposit",
		category: "Income",
		date: new Date("2024-01-15T08:00:00Z"),
		type: "income",
		status: "completed",
		merchant: "Employer Corp",
		tags: ["salary", "income"],
	},
	{
		id: "txn-3",
		accountId: "acc-3",
		amount: -45.99,
		description: "Netflix Subscription",
		category: "Entertainment",
		date: new Date("2024-01-14T12:00:00Z"),
		type: "expense",
		status: "completed",
		merchant: "Netflix",
		tags: ["subscription", "entertainment"],
	},
	{
		id: "txn-4",
		accountId: "acc-1",
		amount: -1200.0,
		description: "Rent Payment",
		category: "Housing",
		date: new Date("2024-01-01T09:00:00Z"),
		type: "expense",
		status: "completed",
		merchant: "Property Management Co",
		tags: ["rent", "housing"],
	},
	{
		id: "txn-5",
		accountId: "acc-2",
		amount: 500.0,
		description: "Transfer from Checking",
		category: "Transfer",
		date: new Date("2024-01-10T15:30:00Z"),
		type: "transfer",
		status: "completed",
		tags: ["savings", "transfer"],
	},
];

// Mock Budgets
export const mockBudgets: Budget[] = [
	{
		id: "budget-1",
		category: "Food & Dining",
		allocated: 600,
		spent: 385.5,
		currency: "USD",
		period: "monthly",
		startDate: new Date("2024-01-01"),
		endDate: new Date("2024-01-31"),
		color: "#ef4444",
	},
	{
		id: "budget-2",
		category: "Transportation",
		allocated: 300,
		spent: 125.0,
		currency: "USD",
		period: "monthly",
		startDate: new Date("2024-01-01"),
		endDate: new Date("2024-01-31"),
		color: "#3b82f6",
	},
	{
		id: "budget-3",
		category: "Entertainment",
		allocated: 200,
		spent: 145.99,
		currency: "USD",
		period: "monthly",
		startDate: new Date("2024-01-01"),
		endDate: new Date("2024-01-31"),
		color: "#10b981",
	},
	{
		id: "budget-4",
		category: "Shopping",
		allocated: 400,
		spent: 220.75,
		currency: "USD",
		period: "monthly",
		startDate: new Date("2024-01-01"),
		endDate: new Date("2024-01-31"),
		color: "#f59e0b",
	},
];

// Mock Spending Categories
export const mockSpendingCategories: SpendingCategory[] = [
	{
		category: "Housing",
		amount: 1200,
		percentage: 35.2,
		color: "#ef4444",
		transactions: 1,
	},
	{
		category: "Food & Dining",
		amount: 385.5,
		percentage: 11.3,
		color: "#3b82f6",
		transactions: 8,
	},
	{
		category: "Transportation",
		amount: 125,
		percentage: 3.7,
		color: "#10b981",
		transactions: 4,
	},
	{
		category: "Entertainment",
		amount: 145.99,
		percentage: 4.3,
		color: "#f59e0b",
		transactions: 3,
	},
	{
		category: "Shopping",
		amount: 220.75,
		percentage: 6.5,
		color: "#8b5cf6",
		transactions: 5,
	},
	{
		category: "Utilities",
		amount: 180,
		percentage: 5.3,
		color: "#06b6d4",
		transactions: 3,
	},
	{
		category: "Healthcare",
		amount: 95,
		percentage: 2.8,
		color: "#84cc16",
		transactions: 2,
	},
	{
		category: "Other",
		amount: 1055.76,
		percentage: 30.9,
		color: "#6b7280",
		transactions: 12,
	},
];

// Mock Financial Goals
export const mockFinancialGoals: FinancialGoal[] = [
	{
		id: "goal-1",
		title: "Emergency Fund",
		targetAmount: 10000,
		currentAmount: 6500,
		deadline: new Date("2024-12-31"),
		category: "Savings",
		priority: "high",
	},
	{
		id: "goal-2",
		title: "Vacation Fund",
		targetAmount: 3000,
		currentAmount: 1200,
		deadline: new Date("2024-07-01"),
		category: "Travel",
		priority: "medium",
	},
	{
		id: "goal-3",
		title: "New Car Down Payment",
		targetAmount: 8000,
		currentAmount: 2800,
		deadline: new Date("2024-09-30"),
		category: "Transportation",
		priority: "medium",
	},
];

// Mock Dashboard Stats
export const mockDashboardStats: DashboardStats = {
	totalBalance: 62270.0,
	monthlyIncome: 6400.0,
	monthlyExpenses: 3407.24,
	savingsRate: 46.8,
	currency: "USD",
};

// Mock Notifications
export const mockNotifications: NotificationItem[] = [
	{
		id: "notif-1",
		type: "warning",
		title: "Budget Alert",
		message: "You have exceeded 80% of your Food & Dining budget",
		timestamp: new Date("2024-01-15T10:00:00Z"),
		read: false,
	},
	{
		id: "notif-2",
		type: "success",
		title: "Goal Achievement",
		message: "Congratulations! You reached 65% of your Emergency Fund goal",
		timestamp: new Date("2024-01-14T16:30:00Z"),
		read: false,
	},
	{
		id: "notif-3",
		type: "info",
		title: "Account Update",
		message: "Your investment account balance has been updated",
		timestamp: new Date("2024-01-14T09:15:00Z"),
		read: true,
	},
];

// Mock Chart Data
export const mockBalanceHistory: ChartDataPoint[] = [
	{ date: "2024-01-01", value: 58500 },
	{ date: "2024-01-02", value: 59200 },
	{ date: "2024-01-03", value: 58800 },
	{ date: "2024-01-04", value: 60100 },
	{ date: "2024-01-05", value: 61500 },
	{ date: "2024-01-06", value: 61200 },
	{ date: "2024-01-07", value: 62000 },
	{ date: "2024-01-08", value: 61800 },
	{ date: "2024-01-09", value: 62500 },
	{ date: "2024-01-10", value: 62200 },
	{ date: "2024-01-11", value: 62800 },
	{ date: "2024-01-12", value: 62600 },
	{ date: "2024-01-13", value: 63100 },
	{ date: "2024-01-14", value: 62900 },
	{ date: "2024-01-15", value: 62270 },
];

export const mockIncomeExpenseHistory: ChartDataPoint[] = [
	{ date: "2024-01-01", value: 6400, label: "Income" },
	{ date: "2024-01-01", value: -3407, label: "Expenses" },
	{ date: "2023-12-01", value: 6400, label: "Income" },
	{ date: "2023-12-01", value: -3850, label: "Expenses" },
	{ date: "2023-11-01", value: 6400, label: "Income" },
	{ date: "2023-11-01", value: -3200, label: "Expenses" },
	{ date: "2023-10-01", value: 6400, label: "Income" },
	{ date: "2023-10-01", value: -3650, label: "Expenses" },
	{ date: "2023-09-01", value: 6400, label: "Income" },
	{ date: "2023-09-01", value: -3100, label: "Expenses" },
	{ date: "2023-08-01", value: 6400, label: "Income" },
	{ date: "2023-08-01", value: -3900, label: "Expenses" },
];

// Mock Navigation Items (will be populated with icons in components)
export const mockNavigationItems: Omit<NavigationItem, "icon">[] = [
	{
		id: "dashboard",
		label: "Dashboard",
		href: "/dashboard",
	},
	{
		id: "accounts",
		label: "Accounts",
		href: "/accounts",
	},
	{
		id: "transactions",
		label: "Transactions",
		href: "/transactions",
	},
	{
		id: "budgets",
		label: "Budgets",
		href: "/budgets",
		badge: "3",
	},
	{
		id: "goals",
		label: "Goals",
		href: "/goals",
	},
	{
		id: "reports",
		label: "Reports",
		href: "/reports",
	},
	{
		id: "settings",
		label: "Settings",
		href: "/settings",
	},
];
