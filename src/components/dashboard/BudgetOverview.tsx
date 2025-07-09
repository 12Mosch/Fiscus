/**
 * Budget Overview Component
 * Displays budget progress with visual indicators and spending alerts
 */

import { AlertTriangle, Target, TrendingUp } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import type { BudgetOverviewProps } from "@/types/dashboard";

export function BudgetOverview({ budgets, className }: BudgetOverviewProps) {
	const formatCurrency = (amount: number, currency: string) => {
		return new Intl.NumberFormat("en-US", {
			style: "currency",
			currency: currency,
			minimumFractionDigits: 0,
			maximumFractionDigits: 0,
		}).format(amount);
	};

	const calculateProgress = (spent: number, allocated: number) => {
		return Math.min((spent / allocated) * 100, 100);
	};

	const getBudgetStatus = (spent: number, allocated: number) => {
		const percentage = (spent / allocated) * 100;

		if (percentage >= 100) {
			return {
				status: "exceeded",
				color: "text-red-600 dark:text-red-400",
				bgColor: "bg-red-500",
			};
		} else if (percentage >= 80) {
			return {
				status: "warning",
				color: "text-yellow-600 dark:text-yellow-400",
				bgColor: "bg-yellow-500",
			};
		} else if (percentage >= 60) {
			return {
				status: "moderate",
				color: "text-blue-600 dark:text-blue-400",
				bgColor: "bg-blue-500",
			};
		} else {
			return {
				status: "good",
				color: "text-green-600 dark:text-green-400",
				bgColor: "bg-green-500",
			};
		}
	};

	const getStatusIcon = (status: string) => {
		switch (status) {
			case "exceeded":
				return <AlertTriangle className="h-4 w-4 text-red-500" />;
			case "warning":
				return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
			default:
				return <Target className="h-4 w-4 text-green-500" />;
		}
	};

	const getStatusBadge = (status: string) => {
		switch (status) {
			case "exceeded":
				return (
					<Badge variant="destructive" className="text-xs">
						Over Budget
					</Badge>
				);
			case "warning":
				return (
					<Badge
						variant="secondary"
						className="text-xs bg-yellow-100 text-yellow-800"
					>
						Near Limit
					</Badge>
				);
			case "moderate":
				return (
					<Badge
						variant="secondary"
						className="text-xs bg-blue-100 text-blue-800"
					>
						On Track
					</Badge>
				);
			default:
				return (
					<Badge
						variant="secondary"
						className="text-xs bg-green-100 text-green-800"
					>
						Good
					</Badge>
				);
		}
	};

	const totalAllocated = budgets.reduce(
		(sum, budget) => sum + budget.allocated,
		0,
	);
	const totalSpent = budgets.reduce((sum, budget) => sum + budget.spent, 0);
	const overallProgress = (totalSpent / totalAllocated) * 100;

	if (budgets.length === 0) {
		return (
			<Card className={className}>
				<CardHeader>
					<CardTitle>Budget Overview</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="flex flex-col items-center justify-center py-8 text-center">
						<div className="rounded-full bg-gray-100 dark:bg-gray-800 p-3 mb-4">
							<Target className="h-6 w-6 text-gray-400" />
						</div>
						<p className="text-gray-500 dark:text-gray-400 mb-2">
							No budgets set up
						</p>
						<Button variant="outline" size="sm">
							Create Budget
						</Button>
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card className={className}>
			<CardHeader className="flex flex-row items-center justify-between">
				<CardTitle>Budget Overview</CardTitle>
				<Button variant="outline" size="sm">
					Manage Budgets
				</Button>
			</CardHeader>
			<CardContent className="space-y-6">
				{/* Overall Budget Summary */}
				<div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4">
					<div className="flex items-center justify-between mb-2">
						<h3 className="text-sm font-medium text-gray-900 dark:text-white">
							Total Monthly Budget
						</h3>
						<TrendingUp className="h-4 w-4 text-gray-400" />
					</div>
					<div className="space-y-2">
						<div className="flex items-center justify-between text-sm">
							<span className="text-gray-600 dark:text-gray-400">
								{formatCurrency(totalSpent, "USD")} of{" "}
								{formatCurrency(totalAllocated, "USD")}
							</span>
							<span
								className={cn(
									"font-medium",
									overallProgress >= 100
										? "text-red-600"
										: overallProgress >= 80
											? "text-yellow-600"
											: "text-green-600",
								)}
							>
								{overallProgress.toFixed(1)}%
							</span>
						</div>
						<Progress value={Math.min(overallProgress, 100)} className="h-2" />
					</div>
				</div>

				{/* Individual Budget Items */}
				<div className="space-y-4">
					{budgets.map((budget) => {
						const progress = calculateProgress(budget.spent, budget.allocated);
						const { status } = getBudgetStatus(budget.spent, budget.allocated);
						const remaining = budget.allocated - budget.spent;

						return (
							<div key={budget.id} className="space-y-3">
								<div className="flex items-center justify-between">
									<div className="flex items-center space-x-2">
										{getStatusIcon(status)}
										<span className="text-sm font-medium text-gray-900 dark:text-white">
											{budget.category}
										</span>
										{getStatusBadge(status)}
									</div>
									<div className="text-right">
										<p className="text-sm font-semibold text-gray-900 dark:text-white">
											{formatCurrency(budget.spent, budget.currency)}
										</p>
										<p className="text-xs text-gray-500 dark:text-gray-400">
											of {formatCurrency(budget.allocated, budget.currency)}
										</p>
									</div>
								</div>

								<div className="space-y-1">
									<Progress value={progress} className="h-2" />
									<div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
										<span>{progress.toFixed(1)}% used</span>
										<span>
											{remaining >= 0
												? `${formatCurrency(remaining, budget.currency)} remaining`
												: `${formatCurrency(Math.abs(remaining), budget.currency)} over budget`}
										</span>
									</div>
								</div>
							</div>
						);
					})}
				</div>

				{/* Quick Actions */}
				<div className="flex space-x-2 pt-4 border-t border-gray-100 dark:border-gray-800">
					<Button variant="outline" size="sm" className="flex-1">
						Add Category
					</Button>
					<Button variant="outline" size="sm" className="flex-1">
						View Details
					</Button>
				</div>
			</CardContent>
		</Card>
	);
}
