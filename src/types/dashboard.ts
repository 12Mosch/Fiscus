/**
 * TypeScript interfaces and types for the Finance Dashboard
 */

export interface Account {
	id: string;
	name: string;
	type: "checking" | "savings" | "credit" | "investment";
	balance: number;
	currency: string;
	lastUpdated: Date;
	accountNumber?: string;
	creditLimit?: number; // For credit accounts - the maximum credit available
}

export interface Transaction {
	id: string;
	accountId: string;
	amount: number;
	description: string;
	category: string;
	date: Date;
	type: "income" | "expense" | "transfer";
	status: "pending" | "completed" | "failed";
	merchant?: string;
	tags?: string[];
}

export interface Budget {
	id: string;
	category: string;
	allocated: number;
	spent: number;
	currency: string;
	period: "monthly" | "weekly" | "yearly";
	startDate: Date;
	endDate: Date;
	color?: string;
}

export interface SpendingCategory {
	category: string;
	amount: number;
	percentage: number;
	color: string;
	transactions: number;
}

export interface ChartDataPoint {
	date: string;
	value: number;
	label?: string;
}

export interface FinancialGoal {
	id: string;
	title: string;
	targetAmount: number;
	currentAmount: number;
	deadline: Date;
	category: string;
	priority: "low" | "medium" | "high";
}

export interface DashboardStats {
	totalBalance: number;
	monthlyIncome: number;
	monthlyExpenses: number;
	savingsRate: number;
	currency: string;
}

export interface NotificationItem {
	id: string;
	type: "info" | "warning" | "error" | "success";
	title: string;
	message: string;
	timestamp: Date;
	read: boolean;
	actionUrl?: string;
}

// Component Props Interfaces
export interface DashboardLayoutProps {
	children: React.ReactNode;
}

export interface FinancialCardProps {
	title: string;
	value: string | number;
	change?: {
		value: number;
		type: "increase" | "decrease";
		period: string;
	};
	icon?: React.ReactNode;
	className?: string;
}

export interface TransactionListProps {
	transactions: Transaction[];
	limit?: number;
	showAccount?: boolean;
	className?: string;
}

export interface BudgetOverviewProps {
	budgets: Budget[];
	className?: string;
}

export interface SpendingChartProps {
	data: SpendingCategory[];
	className?: string;
}

export interface AccountCardProps {
	account: Account;
	className?: string;
}

// Navigation and Layout Types
export interface NavigationItem {
	id: string;
	label: string;
	href: string;
	icon: React.ReactNode;
	badge?: string | number;
	children?: NavigationItem[];
}

export interface SidebarProps {
	navigation: NavigationItem[];
	currentPath: string;
	collapsed?: boolean;
	onToggle?: () => void;
}

export interface HeaderProps {
	user?: {
		name: string;
		email: string;
		avatar?: string;
	};
	notifications?: NotificationItem[];
	onNotificationClick?: (notification: NotificationItem) => void;
}

// Chart and Visualization Types
export interface LineChartProps {
	data: ChartDataPoint[];
	title: string;
	color?: string;
	height?: number;
	className?: string;
}

export interface BarChartProps {
	data: ChartDataPoint[];
	title: string;
	color?: string;
	height?: number;
	className?: string;
}

export interface PieChartProps {
	data: SpendingCategory[];
	title: string;
	height?: number;
	className?: string;
}

// Utility Types
export type TimeRange = "7d" | "30d" | "90d" | "1y" | "all";

export interface DateRangeFilter {
	from: Date;
	to: Date;
}

export interface FilterOptions {
	accounts?: string[];
	categories?: string[];
	dateRange?: DateRangeFilter;
	amountRange?: {
		min: number;
		max: number;
	};
	transactionTypes?: Transaction["type"][];
}

// State Management Types (for Zustand)
export interface DashboardState {
	accounts: Account[];
	transactions: Transaction[];
	budgets: Budget[];
	goals: FinancialGoal[];
	notifications: NotificationItem[];
	selectedTimeRange: TimeRange;
	filters: FilterOptions;
	isLoading: boolean;
	error: string | null;
}

export interface DashboardActions {
	setTimeRange: (range: TimeRange) => void;
	setFilters: (filters: FilterOptions) => void;
	markNotificationAsRead: (id: string) => void;
	clearError: () => void;
	refreshData: () => Promise<void>;
}

export type DashboardStore = DashboardState & DashboardActions;
