/**
 * Bar Chart Placeholder Component
 * Displays a mock bar chart with SVG for financial data comparison
 */

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
	formatCurrencyAbbreviated,
	formatCurrencyCompact,
} from "@/lib/formatters";
import type { BarChartProps } from "@/types/dashboard";

export function BarChart({
	data,
	title,
	color = "#3b82f6",
	height = 250,
	className,
}: BarChartProps) {
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

	// Calculate chart dimensions
	const padding = 40;
	const chartWidth = 400;
	const chartHeight = height - padding * 2;
	const maxBarWidth = (chartWidth * 0.8) / data.length;
	const barWidth = Math.max(8, Math.min(maxBarWidth, 40));
	const barSpacing = Math.max(
		4,
		data.length > 1
			? (chartWidth * 0.8 - barWidth * data.length) / (data.length - 1)
			: 4,
	);
	const values = data.map((d) => Math.abs(d.value));
	const maxValue = Math.max(...values) || 1;

	// Generate bars
	const generateBars = () => {
		return data.map((point, index) => {
			const barHeight = (Math.abs(point.value) / maxValue) * chartHeight;
			const x = index * (barWidth + barSpacing);
			const y = chartHeight - barHeight;
			const isNegative = point.value < 0;

			return {
				...point,
				x,
				y: isNegative ? chartHeight : y,
				width: barWidth,
				height: barHeight,
				isNegative,
			};
		});
	};

	const bars = generateBars();

	return (
		<Card className={className}>
			<CardHeader>
				<CardTitle className="text-base font-semibold">{title}</CardTitle>
			</CardHeader>

			<CardContent>
				<div className="relative" style={{ height }}>
					<svg
						width="100%"
						height={height}
						viewBox={`0 0 ${chartWidth + padding * 2} ${height}`}
						className="overflow-visible"
						role="img"
						aria-labelledby={`bar-chart-title-${title.replace(/\s+/g, "-")}`}
					>
						<title id={`bar-chart-title-${title.replace(/\s+/g, "-")}`}>
							{title} - Bar chart showing financial data over time
						</title>
						{/* Grid Lines */}
						<g transform={`translate(${padding}, ${padding})`}>
							{/* Horizontal grid lines */}
							{[0, 0.25, 0.5, 0.75, 1].map((ratio) => (
								<line
									key={`h-grid-${ratio}`}
									x1="0"
									y1={ratio * chartHeight}
									x2={chartWidth}
									y2={ratio * chartHeight}
									stroke="#e5e7eb"
									strokeWidth="1"
									strokeDasharray="2,2"
									className="dark:stroke-gray-700"
								/>
							))}

							{/* Zero line (if there are negative values) */}
							{data.some((d) => d.value < 0) && (
								<line
									x1="0"
									y1={chartHeight}
									x2={chartWidth}
									y2={chartHeight}
									stroke="#374151"
									strokeWidth="2"
									className="dark:stroke-gray-400"
								/>
							)}
						</g>

						{/* Bars */}
						<g transform={`translate(${padding}, ${padding})`}>
							{bars.map((bar) => (
								<g key={`bar-${bar.date}`}>
									<rect
										x={bar.x}
										y={bar.y}
										width={bar.width}
										height={bar.height}
										fill={bar.isNegative ? "#ef4444" : color}
										rx="2"
										className="hover:opacity-80 transition-opacity cursor-pointer"
									>
										<title>{`${bar.date}: ${formatCurrencyCompact(bar.value)}`}</title>
									</rect>

									{/* Value labels on top of bars */}
									<text
										x={bar.x + bar.width / 2}
										y={bar.isNegative ? bar.y + bar.height + 15 : bar.y - 5}
										textAnchor="middle"
										className="text-xs fill-gray-600 dark:fill-gray-400"
									>
										{formatCurrencyAbbreviated(bar.value)}
									</text>
								</g>
							))}
						</g>

						{/* Y-axis labels */}
						<g transform={`translate(${padding - 10}, ${padding})`}>
							{[0, 0.25, 0.5, 0.75, 1].map((ratio) => {
								const value = maxValue * (1 - ratio);
								const y = ratio * chartHeight;

								return (
									<text
										key={`y-label-${ratio}`}
										x="0"
										y={y}
										textAnchor="end"
										dominantBaseline="middle"
										className="text-xs fill-gray-500 dark:fill-gray-400"
									>
										{formatCurrencyAbbreviated(value)}
									</text>
								);
							})}
						</g>

						{/* X-axis labels */}
						<g transform={`translate(${padding}, ${height - padding + 15})`}>
							{bars.map((bar) => {
								const date = new Date(bar.date);
								const label =
									bar.label ||
									date.toLocaleDateString("en-US", {
										month: "short",
										...(data.length <= 12 ? { day: "numeric" } : {}),
									});

								return (
									<text
										key={`x-label-${bar.date}`}
										x={bar.x + bar.width / 2}
										y="0"
										textAnchor="middle"
										className="text-xs fill-gray-500 dark:fill-gray-400"
									>
										{label}
									</text>
								);
							})}
						</g>
					</svg>
				</div>

				{/* Summary Statistics */}
				<div className="mt-4 grid grid-cols-3 gap-4 text-center border-t border-gray-100 dark:border-gray-800 pt-4">
					<div>
						<p className="text-xs text-gray-500 dark:text-gray-400">Average</p>
						<p className="text-sm font-semibold text-gray-900 dark:text-white">
							{formatCurrencyAbbreviated(
								data.reduce((sum, d) => sum + d.value, 0) / data.length,
							)}
						</p>
					</div>
					<div>
						<p className="text-xs text-gray-500 dark:text-gray-400">Highest</p>
						<p className="text-sm font-semibold text-gray-900 dark:text-white">
							{formatCurrencyAbbreviated(Math.max(...data.map((d) => d.value)))}
						</p>
					</div>
					<div>
						<p className="text-xs text-gray-500 dark:text-gray-400">Lowest</p>
						<p className="text-sm font-semibold text-gray-900 dark:text-white">
							{formatCurrencyAbbreviated(Math.min(...data.map((d) => d.value)))}
						</p>
					</div>
				</div>
			</CardContent>
		</Card>
	);
}
