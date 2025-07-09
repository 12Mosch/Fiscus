/**
 * Pie Chart Placeholder Component
 * Displays a mock pie chart with SVG for spending category visualization
 */

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrencyCompact } from "@/lib/formatters";
import type { PieChartProps } from "@/types/dashboard";

export function PieChart({
	data,
	title,
	height = 300,
	className,
}: PieChartProps) {
	if (!data || data.length === 0) {
		return (
			<Card className={className}>
				<CardHeader>
					<CardTitle>{title}</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="flex items-center justify-center" style={{ height }}>
						<p className="text-gray-500 dark:text-gray-400">
							No data available
						</p>
					</div>
				</CardContent>
			</Card>
		);
	}

	const radius = 80;
	const centerX = 120;
	const centerY = 120;
	const total = data.reduce((sum, item) => sum + item.amount, 0);

	// Generate pie slices
	const generateSlices = () => {
		let currentAngle = -90; // Start from top

		return data.map((item, _index) => {
			const percentage = (item.amount / total) * 100;
			const angle = (item.amount / total) * 360;
			const startAngle = currentAngle;
			const endAngle = currentAngle + angle;

			// Convert to radians
			const startRad = (startAngle * Math.PI) / 180;
			const endRad = (endAngle * Math.PI) / 180;

			// Calculate arc coordinates
			const x1 = centerX + radius * Math.cos(startRad);
			const y1 = centerY + radius * Math.sin(startRad);
			const x2 = centerX + radius * Math.cos(endRad);
			const y2 = centerY + radius * Math.sin(endRad);

			const largeArcFlag = angle > 180 ? 1 : 0;

			const pathData = [
				`M ${centerX} ${centerY}`,
				`L ${x1} ${y1}`,
				`A ${radius} ${radius} 0 ${largeArcFlag} 1 ${x2} ${y2}`,
				"Z",
			].join(" ");

			currentAngle += angle;

			return {
				...item,
				pathData,
				percentage,
				startAngle,
				endAngle,
			};
		});
	};

	const slices = generateSlices();

	return (
		<Card className={className}>
			<CardHeader>
				<CardTitle>{title}</CardTitle>
				<div className="text-sm text-gray-600 dark:text-gray-400">
					Total: {formatCurrencyCompact(total)}
				</div>
			</CardHeader>

			<CardContent>
				<div className="flex flex-col lg:flex-row items-start gap-6">
					{/* Pie Chart */}
					<div className="flex-shrink-0">
						<svg
							width="240"
							height="240"
							className="overflow-visible"
							role="img"
							aria-labelledby={`pie-chart-title-${title.replace(/\s+/g, "-")}`}
						>
							<title id={`pie-chart-title-${title.replace(/\s+/g, "-")}`}>
								{title} - Pie chart showing spending breakdown by category
							</title>
							{/* Pie slices */}
							{slices.map((slice, _index) => (
								<g key={slice.category}>
									<path
										d={slice.pathData}
										fill={slice.color}
										stroke="white"
										strokeWidth="2"
										className="hover:opacity-80 transition-opacity cursor-pointer"
									>
										<title>
											{slice.category}: {formatCurrencyCompact(slice.amount)} (
											{slice.percentage.toFixed(1)}%)
										</title>
									</path>
								</g>
							))}

							{/* Center circle for donut effect */}
							<circle
								cx={centerX}
								cy={centerY}
								r="30"
								fill="white"
								stroke="#e5e7eb"
								strokeWidth="1"
								className="dark:fill-gray-950 dark:stroke-gray-700"
							/>

							{/* Center text */}
							<text
								x={centerX}
								y={centerY - 5}
								textAnchor="middle"
								className="text-xs font-medium fill-gray-900 dark:fill-white"
							>
								Total
							</text>
							<text
								x={centerX}
								y={centerY + 8}
								textAnchor="middle"
								className="text-xs fill-gray-600 dark:fill-gray-400"
							>
								{formatCurrencyCompact(total)}
							</text>
						</svg>
					</div>

					{/* Legend */}
					<div className="flex-1 space-y-3">
						<h4 className="text-sm font-medium text-gray-900 dark:text-white mb-3">
							Spending Breakdown
						</h4>
						<div className="space-y-2 max-h-48 overflow-y-auto">
							{slices.map((slice, _index) => (
								<div
									key={slice.category}
									className="flex items-center justify-between py-1"
								>
									<div className="flex items-center space-x-3">
										<div
											className="w-3 h-3 rounded-full flex-shrink-0"
											style={{ backgroundColor: slice.color }}
										/>
										<div className="min-w-0 flex-1">
											<p className="text-sm font-medium text-gray-900 dark:text-white truncate">
												{slice.category}
											</p>
											<p className="text-xs text-gray-500 dark:text-gray-400">
												{slice.transactions} transaction
												{slice.transactions !== 1 ? "s" : ""}
											</p>
										</div>
									</div>
									<div className="text-right flex-shrink-0">
										<p className="text-sm font-semibold text-gray-900 dark:text-white">
											{formatCurrencyCompact(slice.amount)}
										</p>
										<Badge variant="secondary" className="text-xs">
											{slice.percentage.toFixed(1)}%
										</Badge>
									</div>
								</div>
							))}
						</div>

						{/* Summary Stats */}
						<div className="pt-3 border-t border-gray-100 dark:border-gray-800">
							<div className="grid grid-cols-2 gap-4 text-center">
								<div>
									<p className="text-xs text-gray-500 dark:text-gray-400">
										Categories
									</p>
									<p className="text-lg font-semibold text-gray-900 dark:text-white">
										{data.length}
									</p>
								</div>
								<div>
									<p className="text-xs text-gray-500 dark:text-gray-400">
										Transactions
									</p>
									<p className="text-lg font-semibold text-gray-900 dark:text-white">
										{data.reduce((sum, item) => sum + item.transactions, 0)}
									</p>
								</div>
							</div>
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	);
}
