/**
 * Account Card Component
 * Displays individual account information with balance and details
 */

import { CreditCard, PiggyBank, TrendingUp, Wallet } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrency, formatRelativeDate } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import type { Account, AccountCardProps } from "@/types/dashboard";

const accountTypeIcons = {
	checking: <Wallet className="h-5 w-5" />,
	savings: <PiggyBank className="h-5 w-5" />,
	credit: <CreditCard className="h-5 w-5" />,
	investment: <TrendingUp className="h-5 w-5" />,
};

const accountTypeColors = {
	checking: "bg-blue-500",
	savings: "bg-green-500",
	credit: "bg-red-500",
	investment: "bg-purple-500",
};

const accountTypeLabels = {
	checking: "Checking",
	savings: "Savings",
	credit: "Credit",
	investment: "Investment",
};

export function AccountCard({ account, className }: AccountCardProps) {
	const getBalanceColor = (balance: number, type: Account["type"]) => {
		if (type === "credit") {
			// For credit accounts, negative balance means debt (red)
			// Positive balance means credit (green)
			return balance < 0
				? "text-red-600 dark:text-red-400"
				: "text-green-600 dark:text-green-400";
		}
		return balance >= 0
			? "text-gray-900 dark:text-white"
			: "text-red-600 dark:text-red-400";
	};

	return (
		<Card className={cn("transition-all hover:shadow-md", className)}>
			<CardHeader className="pb-3">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						<div
							className={cn(
								"flex h-10 w-10 items-center justify-center rounded-lg text-white",
								accountTypeColors[account.type],
							)}
						>
							{accountTypeIcons[account.type]}
						</div>
						<div>
							<CardTitle className="text-base font-semibold">
								{account.name}
							</CardTitle>
							<div className="flex items-center space-x-2 mt-1">
								<Badge variant="secondary" className="text-xs">
									{accountTypeLabels[account.type]}
								</Badge>
								{account.accountNumber && (
									<span className="text-xs text-gray-500 dark:text-gray-400">
										{account.accountNumber}
									</span>
								)}
							</div>
						</div>
					</div>
				</div>
			</CardHeader>

			<CardContent className="pt-0">
				<div className="space-y-3">
					{/* Balance */}
					<div>
						<p className="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Current Balance
						</p>
						<p
							className={cn(
								"text-2xl font-bold",
								getBalanceColor(account.balance, account.type),
							)}
						>
							{formatCurrency(account.balance, account.currency, {
								handleNegative: true,
							})}
						</p>
					</div>

					{/* Last Updated */}
					<div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
						<span>Last updated</span>
						<span>{formatRelativeDate(account.lastUpdated)}</span>
					</div>

					{/* Account Type Specific Info */}
					{account.type === "credit" && (
						<div className="pt-2 border-t border-gray-100 dark:border-gray-800">
							<div className="flex justify-between text-xs">
								<span className="text-gray-500 dark:text-gray-400">
									Available Credit
								</span>
								<span className="font-medium text-green-600 dark:text-green-400">
									{formatCurrency(Math.abs(account.balance), account.currency)}
								</span>
							</div>
						</div>
					)}

					{account.type === "investment" && (
						<div className="pt-2 border-t border-gray-100 dark:border-gray-800">
							<div className="flex justify-between text-xs">
								<span className="text-gray-500 dark:text-gray-400">
									Portfolio Value
								</span>
								<span className="font-medium text-purple-600 dark:text-purple-400">
									{formatCurrency(account.balance, account.currency)}
								</span>
							</div>
						</div>
					)}
				</div>
			</CardContent>
		</Card>
	);
}
