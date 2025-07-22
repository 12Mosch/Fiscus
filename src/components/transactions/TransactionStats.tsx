/**
 * Transaction Statistics Component
 * Displays transaction statistics and summary information
 */

import { Activity, TrendingDown, TrendingUp } from "lucide-react";
import { useEffect } from "react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { useUserId } from "@/stores/auth-store";
import { useTransactionsStore } from "@/stores/transactions-store";
import type { TransactionFilters } from "@/types/api";
import { formatCurrency } from "@/utils/currency";

interface TransactionStatsProps {
	/** Filters to apply when loading stats */
	filters?: Partial<TransactionFilters>;
	/** Whether to show detailed breakdown */
	showDetails?: boolean;
}

export function TransactionStats({
	filters,
	showDetails = true,
}: TransactionStatsProps) {
	const userId = useUserId();
	const { stats, loadingStates, loadStats } = useTransactionsStore();

	// Load stats when component mounts or filters change
	useEffect(() => {
		if (userId) {
			const statsFilters: TransactionFilters = {
				user_id: userId,
				...filters,
			};
			loadStats(statsFilters);
		}
	}, [userId, filters, loadStats]);

	if (loadingStates.stats) {
		return (
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
				{Array.from({ length: 4 }, (_, index) => `stats-loading-${index}`).map(
					(key) => (
						<Card key={key}>
							<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
								<Skeleton className="h-4 w-20" />
								<Skeleton className="h-4 w-4" />
							</CardHeader>
							<CardContent>
								<Skeleton className="h-8 w-24 mb-2" />
								<Skeleton className="h-3 w-16" />
							</CardContent>
						</Card>
					),
				)}
			</div>
		);
	}

	if (!stats) {
		return (
			<Card>
				<CardContent className="p-6">
					<div className="text-center text-muted-foreground">
						No transaction statistics available
					</div>
				</CardContent>
			</Card>
		);
	}

	const netIncomeColor =
		stats.net_income >= 0 ? "text-green-600" : "text-red-600";
	const netIncomeIcon = stats.net_income >= 0 ? TrendingUp : TrendingDown;
	const NetIncomeIcon = netIncomeIcon;

	return (
		<div className="space-y-6">
			{/* Main Stats Cards */}
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
				{/* Total Income */}
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Total Income</CardTitle>
						<TrendingUp className="h-4 w-4 text-green-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-green-600">
							{formatCurrency(stats.total_income)}
						</div>
						<p className="text-xs text-muted-foreground">
							{stats.transactions_by_type.income || 0} transactions
						</p>
					</CardContent>
				</Card>

				{/* Total Expenses */}
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Total Expenses
						</CardTitle>
						<TrendingDown className="h-4 w-4 text-red-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold text-red-600">
							{formatCurrency(stats.total_expenses)}
						</div>
						<p className="text-xs text-muted-foreground">
							{stats.transactions_by_type.expense || 0} transactions
						</p>
					</CardContent>
				</Card>

				{/* Net Income */}
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Net Income</CardTitle>
						<NetIncomeIcon className={`h-4 w-4 ${netIncomeColor}`} />
					</CardHeader>
					<CardContent>
						<div className={`text-2xl font-bold ${netIncomeColor}`}>
							{formatCurrency(stats.net_income)}
						</div>
						<p className="text-xs text-muted-foreground">Income - Expenses</p>
					</CardContent>
				</Card>

				{/* Total Transactions */}
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Total Transactions
						</CardTitle>
						<Activity className="h-4 w-4 text-blue-600" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">{stats.total_transactions}</div>
						<p className="text-xs text-muted-foreground">
							Avg: {formatCurrency(stats.average_transaction_amount)}
						</p>
					</CardContent>
				</Card>
			</div>

			{/* Detailed Breakdown */}
			{showDetails && (
				<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
					{/* Transaction Types Breakdown */}
					<Card>
						<CardHeader>
							<CardTitle className="text-lg">Transaction Types</CardTitle>
						</CardHeader>
						<CardContent>
							<div className="space-y-3">
								{Object.entries(stats.transactions_by_type).map(
									([type, count]) => (
										<div
											key={type}
											className="flex items-center justify-between"
										>
											<div className="flex items-center gap-2">
												<Badge
													variant={
														type === "income"
															? "default"
															: type === "expense"
																? "destructive"
																: "secondary"
													}
												>
													{type.charAt(0).toUpperCase() + type.slice(1)}
												</Badge>
											</div>
											<div className="text-sm font-medium">
												{count} transactions
											</div>
										</div>
									),
								)}
							</div>
						</CardContent>
					</Card>

					{/* Transaction Status Breakdown */}
					<Card>
						<CardHeader>
							<CardTitle className="text-lg">Transaction Status</CardTitle>
						</CardHeader>
						<CardContent>
							<div className="space-y-3">
								{Object.entries(stats.transactions_by_status).map(
									([status, count]) => (
										<div
											key={status}
											className="flex items-center justify-between"
										>
											<div className="flex items-center gap-2">
												<Badge
													variant={
														status === "completed"
															? "default"
															: status === "pending"
																? "secondary"
																: "destructive"
													}
												>
													{status.charAt(0).toUpperCase() + status.slice(1)}
												</Badge>
											</div>
											<div className="text-sm font-medium">
												{count} transactions
											</div>
										</div>
									),
								)}
							</div>
						</CardContent>
					</Card>

					{/* Largest Transactions */}
					{(stats.largest_income || stats.largest_expense) && (
						<Card className="lg:col-span-2">
							<CardHeader>
								<CardTitle className="text-lg">Largest Transactions</CardTitle>
							</CardHeader>
							<CardContent>
								<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
									{stats.largest_income && (
										<div className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-950 rounded-lg">
											<div>
												<div className="text-sm font-medium text-green-800 dark:text-green-200">
													Largest Income
												</div>
												<div className="text-lg font-bold text-green-600">
													{formatCurrency(stats.largest_income)}
												</div>
											</div>
											<TrendingUp className="h-6 w-6 text-green-600" />
										</div>
									)}

									{stats.largest_expense && (
										<div className="flex items-center justify-between p-3 bg-red-50 dark:bg-red-950 rounded-lg">
											<div>
												<div className="text-sm font-medium text-red-800 dark:text-red-200">
													Largest Expense
												</div>
												<div className="text-lg font-bold text-red-600">
													{formatCurrency(stats.largest_expense)}
												</div>
											</div>
											<TrendingDown className="h-6 w-6 text-red-600" />
										</div>
									)}
								</div>
							</CardContent>
						</Card>
					)}
				</div>
			)}
		</div>
	);
}
