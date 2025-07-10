/**
 * Financial Card Component
 * Reusable card for displaying financial metrics with optional change indicators
 */

import { TrendingDown, TrendingUp } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrency } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import type { FinancialCardProps } from "@/types/dashboard";

export function FinancialCard({
	title,
	value,
	change,
	icon,
	className,
}: FinancialCardProps) {
	const formatValue = (val: string | number) => {
		if (typeof val === "number") {
			return formatCurrency(val, "USD", {
				minimumFractionDigits: 0,
				maximumFractionDigits: 2,
			});
		}
		return val;
	};

	const formatChange = (changeValue: number) => {
		const formatted = Math.abs(changeValue).toFixed(1);
		return changeValue === 0
			? `${formatted}%`
			: `${changeValue >= 0 ? "+" : "-"}${formatted}%`;
	};

	return (
		<Card className={cn("transition-all hover:shadow-md", className)}>
			<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle className="text-sm font-medium text-gray-600 dark:text-gray-400">
					{title}
				</CardTitle>
				{icon && <div className="text-gray-400 dark:text-gray-500">{icon}</div>}
			</CardHeader>
			<CardContent>
				<div className="text-2xl font-bold text-gray-900 dark:text-white">
					{formatValue(value)}
				</div>
				{change && (
					<div className="mt-2 flex items-center text-xs">
						{change.type === "increase" ? (
							<TrendingUp
								className="mr-1 h-3 w-3 text-green-500"
								aria-hidden="true"
							/>
						) : (
							<TrendingDown
								className="mr-1 h-3 w-3 text-red-500"
								aria-hidden="true"
							/>
						)}
						<span
							className={cn(
								"font-medium",
								change.type === "increase" ? "text-green-600" : "text-red-600",
							)}
						>
							{formatChange(change.value)}
						</span>
						<span className="ml-1 text-gray-500">from {change.period}</span>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
