/**
 * Main Dashboard Page Component
 * Combines all dashboard widgets and charts into a comprehensive financial overview
 */

import {
	Activity,
	CreditCard,
	DollarSign,
	PiggyBank,
	Target,
	TrendingDown,
	TrendingUp,
} from "lucide-react";
import {
	mockAccounts,
	mockBalanceHistory,
	mockBudgets,
	mockDashboardStats,
	mockIncomeExpenseHistory,
	mockSpendingCategories,
	mockTransactions,
} from "@/data/mockData";
import { BarChart } from "../charts/BarChart";
import { LineChart } from "../charts/LineChart";
import { PieChart } from "../charts/PieChart";
import { AccountCard } from "./AccountCard";
import { BudgetOverview } from "./BudgetOverview";
import { FinancialCard } from "./FinancialCard";
import { TransactionList } from "./TransactionList";

export function DashboardPage() {
	const stats = mockDashboardStats;

	// Calculate additional metrics
	const netWorth = stats.totalBalance;
	const monthlyNet = stats.monthlyIncome - stats.monthlyExpenses;
	const expenseRatio = (stats.monthlyExpenses / stats.monthlyIncome) * 100;

	// Calculate budget summary dynamically
	const budgetSummary = mockBudgets.reduce(
		(acc, budget) => {
			const spent = (budget.spent / budget.allocated) * 100;
			if (spent > 100) {
				acc.overLimit++;
			} else if (spent > 90) {
				acc.warning++;
			} else {
				acc.onTrack++;
			}
			return acc;
		},
		{ onTrack: 0, warning: 0, overLimit: 0 },
	);

	return (
		<div className="space-y-6">
			{/* Page Header */}
			<div className="flex flex-col sm:flex-row sm:items-center sm:justify-between">
				<div className="text-left">
					<h1 className="text-2xl font-bold text-gray-900 dark:text-white !text-left">
						Dashboard
					</h1>
					<p className="text-gray-600 dark:text-gray-400 mt-1 text-left">
						Welcome back! Here's your financial overview.
					</p>
				</div>
				<div className="mt-4 sm:mt-0">
					<p className="text-sm text-gray-500 dark:text-gray-400">
						Last updated: {new Date().toLocaleString()}
					</p>
				</div>
			</div>

			{/* Key Metrics Cards */}
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
				<FinancialCard
					title="Total Balance"
					value={stats.totalBalance}
					change={{
						value: 2.5,
						type: "increase",
						period: "last month",
					}}
					icon={<DollarSign className="h-5 w-5" />}
				/>

				<FinancialCard
					title="Monthly Income"
					value={stats.monthlyIncome}
					change={{
						value: 1.2,
						type: "increase",
						period: "last month",
					}}
					icon={<TrendingUp className="h-5 w-5" />}
				/>

				<FinancialCard
					title="Monthly Expenses"
					value={stats.monthlyExpenses}
					change={{
						value: 3.1,
						type: "decrease",
						period: "last month",
					}}
					icon={<TrendingDown className="h-5 w-5" />}
				/>

				<FinancialCard
					title="Savings Rate"
					value={`${stats.savingsRate}%`}
					change={{
						value: 4.2,
						type: "increase",
						period: "last month",
					}}
					icon={<PiggyBank className="h-5 w-5" />}
				/>
			</div>

			{/* Charts Row */}
			<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
				<LineChart
					data={mockBalanceHistory}
					title="Account Balance Trend"
					color="#3b82f6"
					height={300}
				/>

				<BarChart
					data={mockIncomeExpenseHistory
						.filter((d) => d.label === "Income")
						.slice(0, 6)}
					title="Monthly Income vs Expenses"
					color="#10b981"
					height={300}
				/>
			</div>

			{/* Main Content Grid */}
			<div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
				{/* Left Column - Accounts and Transactions */}
				<div className="lg:col-span-2 space-y-6">
					{/* Account Cards */}
					<div>
						<h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
							Your Accounts
						</h2>
						<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
							{mockAccounts.slice(0, 4).map((account) => (
								<AccountCard key={account.id} account={account} />
							))}
						</div>
					</div>

					{/* Recent Transactions */}
					<TransactionList
						transactions={mockTransactions}
						limit={8}
						showAccount={true}
					/>
				</div>

				{/* Right Column - Budget and Spending */}
				<div className="space-y-6">
					{/* Budget Overview */}
					<BudgetOverview budgets={mockBudgets} />

					{/* Spending Breakdown */}
					<PieChart
						data={mockSpendingCategories}
						title="Spending by Category"
						height={400}
					/>
				</div>
			</div>

			{/* Additional Insights Row */}
			<div className="grid grid-cols-1 md:grid-cols-3 gap-6">
				{/* Quick Stats */}
				<div className="bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
						Quick Insights
					</h3>
					<div className="space-y-4">
						<div className="flex items-center justify-between">
							<span className="text-sm text-gray-600 dark:text-gray-400">
								Net Worth
							</span>
							<span className="text-sm font-semibold text-gray-900 dark:text-white">
								${netWorth.toLocaleString()}
							</span>
						</div>
						<div className="flex items-center justify-between">
							<span className="text-sm text-gray-600 dark:text-gray-400">
								Monthly Net
							</span>
							<span
								className={`text-sm font-semibold ${
									monthlyNet >= 0 ? "text-green-600" : "text-red-600"
								}`}
							>
								${monthlyNet.toLocaleString()}
							</span>
						</div>
						<div className="flex items-center justify-between">
							<span className="text-sm text-gray-600 dark:text-gray-400">
								Expense Ratio
							</span>
							<span className="text-sm font-semibold text-gray-900 dark:text-white">
								{expenseRatio.toFixed(1)}%
							</span>
						</div>
					</div>
				</div>

				{/* Goals Progress */}
				<div className="bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
						Financial Goals
					</h3>
					<div className="space-y-3">
						<div className="flex items-center space-x-3">
							<Target className="h-4 w-4 text-blue-500" />
							<div className="flex-1">
								<p className="text-sm font-medium text-gray-900 dark:text-white">
									Emergency Fund
								</p>
								<div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mt-1">
									<div
										className="bg-blue-500 h-2 rounded-full"
										style={{ width: "65%" }}
									></div>
								</div>
								<p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
									65% complete
								</p>
							</div>
						</div>
						<div className="flex items-center space-x-3">
							<Target className="h-4 w-4 text-green-500" />
							<div className="flex-1">
								<p className="text-sm font-medium text-gray-900 dark:text-white">
									Vacation Fund
								</p>
								<div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mt-1">
									<div
										className="bg-green-500 h-2 rounded-full"
										style={{ width: "40%" }}
									></div>
								</div>
								<p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
									40% complete
								</p>
							</div>
						</div>
					</div>
				</div>

				{/* Activity Summary */}
				<div className="bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
						This Month
					</h3>
					<div className="space-y-4">
						<div className="flex items-center space-x-3">
							<Activity className="h-4 w-4 text-blue-500" />
							<div>
								<p className="text-sm font-medium text-gray-900 dark:text-white">
									{mockTransactions.length} Transactions
								</p>
								<p className="text-xs text-gray-500 dark:text-gray-400">
									Across {mockAccounts.length} accounts
								</p>
							</div>
						</div>
						<div className="flex items-center space-x-3">
							<CreditCard className="h-4 w-4 text-green-500" />
							<div>
								<p className="text-sm font-medium text-gray-900 dark:text-white">
									{mockBudgets.length} Active Budgets
								</p>
								<p className="text-xs text-gray-500 dark:text-gray-400">
									{budgetSummary.onTrack} on track, {budgetSummary.overLimit}{" "}
									over limit
								</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
