/**
 * Transaction List Component
 * Displays a list of recent transactions with filtering and formatting
 */

import {
	ArrowDownLeft,
	ArrowRightLeft,
	ArrowUpRight,
	CheckCircle,
	Clock,
	XCircle,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { cn } from "@/lib/utils";
import type { Transaction, TransactionListProps } from "@/types/dashboard";

const transactionTypeIcons = {
	income: <ArrowDownLeft className="h-4 w-4 text-green-500" />,
	expense: <ArrowUpRight className="h-4 w-4 text-red-500" />,
	transfer: <ArrowRightLeft className="h-4 w-4 text-blue-500" />,
};

const statusIcons = {
	completed: <CheckCircle className="h-3 w-3 text-green-500" />,
	pending: <Clock className="h-3 w-3 text-yellow-500" />,
	failed: <XCircle className="h-3 w-3 text-red-500" />,
};

export function TransactionList({
	transactions,
	limit = 10,
	showAccount = false,
	className,
}: TransactionListProps) {
	const displayTransactions = transactions.slice(0, limit);

	const formatAmount = (amount: number, type: Transaction["type"]) => {
		const formatted = new Intl.NumberFormat("en-US", {
			style: "currency",
			currency: "USD",
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(Math.abs(amount));

		if (type === "income") {
			return `+${formatted}`;
		} else if (type === "expense") {
			return `-${formatted}`;
		}
		return formatted;
	};

	const getAmountColor = (type: Transaction["type"]) => {
		switch (type) {
			case "income":
				return "text-green-600 dark:text-green-400";
			case "expense":
				return "text-red-600 dark:text-red-400";
			default:
				return "text-gray-900 dark:text-white";
		}
	};

	const formatDate = (date: Date) => {
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

	const getCategoryColor = (category: string) => {
		// Simple hash function to generate consistent colors for categories
		let hash = 0;
		for (let i = 0; i < category.length; i++) {
			hash = category.charCodeAt(i) + ((hash << 5) - hash);
		}
		const colors = [
			"bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300",
			"bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300",
			"bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300",
			"bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300",
			"bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-300",
			"bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-300",
		];
		return colors[Math.abs(hash) % colors.length];
	};

	if (displayTransactions.length === 0) {
		return (
			<Card className={className}>
				<CardHeader>
					<CardTitle>Recent Transactions</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="flex flex-col items-center justify-center py-8 text-center">
						<div className="rounded-full bg-gray-100 dark:bg-gray-800 p-3 mb-4">
							<ArrowRightLeft className="h-6 w-6 text-gray-400" />
						</div>
						<p className="text-gray-500 dark:text-gray-400">
							No transactions found
						</p>
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card className={className}>
			<CardHeader className="flex flex-row items-center justify-between">
				<CardTitle>Recent Transactions</CardTitle>
				<Button variant="outline" size="sm">
					View All
				</Button>
			</CardHeader>
			<CardContent className="p-0">
				<div className="divide-y divide-gray-100 dark:divide-gray-800">
					{displayTransactions.map((transaction, _index) => (
						<div
							key={transaction.id}
							className="p-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors"
						>
							<div className="flex items-center justify-between">
								<div className="flex items-center space-x-3">
									{/* Transaction Type Icon */}
									<div className="flex-shrink-0">
										{transactionTypeIcons[transaction.type]}
									</div>

									{/* Transaction Details */}
									<div className="min-w-0 flex-1">
										<div className="flex items-center space-x-2">
											<p className="text-sm font-medium text-gray-900 dark:text-white truncate">
												{transaction.description}
											</p>
											<div className="flex items-center space-x-1">
												{statusIcons[transaction.status]}
											</div>
										</div>

										<div className="flex items-center space-x-2 mt-1">
											<Badge
												variant="secondary"
												className={cn(
													"text-xs",
													getCategoryColor(transaction.category),
												)}
											>
												{transaction.category}
											</Badge>
											{transaction.merchant && (
												<span className="text-xs text-gray-500 dark:text-gray-400">
													• {transaction.merchant}
												</span>
											)}
										</div>

										<p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
											{formatDate(transaction.date)}
										</p>
									</div>
								</div>

								{/* Amount */}
								<div className="text-right">
									<p
										className={cn(
											"text-sm font-semibold",
											getAmountColor(transaction.type),
										)}
									>
										{formatAmount(transaction.amount, transaction.type)}
									</p>
									{showAccount && (
										<p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
											Account: {transaction.accountId}
										</p>
									)}
								</div>
							</div>
						</div>
					))}
				</div>

				{transactions.length > limit && (
					<div className="p-4 border-t border-gray-100 dark:border-gray-800">
						<Button variant="ghost" className="w-full" size="sm">
							Show {transactions.length - limit} more transactions
						</Button>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
