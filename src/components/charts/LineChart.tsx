/**
 * Line Chart Placeholder Component
 * Displays a mock line chart with SVG for financial data visualization
 */

import { TrendingDown, TrendingUp } from "lucide-react";
import { useId } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrencyCompact } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import type { LineChartProps } from "@/types/dashboard";

export function LineChart({
	data,
	title,
	color = "#3b82f6",
	height = 200,
	className,
}: LineChartProps) {
	// Generate unique IDs for accessibility and SVG elements
	const chartId = useId();
	const titleId = `line-chart-title-${chartId}`;
	const gradientId = `gradient-${chartId}`;

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

	// Calculate chart dimensions and scaling
	const padding = 40;
	const chartWidth = 400;
	const chartHeight = height - padding * 2;

	const values = data.map((d) => d.value);
	const minValue = Math.min(...values);
	const maxValue = Math.max(...values);
	const valueRange = maxValue - minValue || Math.max(1, Math.abs(maxValue) * 0.1);

	// Generate SVG path for the line
	const generatePath = () => {
		const points = data.map((point, index) => {
			const x = (index / (data.length - 1)) * chartWidth;
			const y =
				chartHeight - ((point.value - minValue) / valueRange) * chartHeight;
			return `${x},${y}`;
		});

		return `M ${points.join(" L ")}`;
	};

	// Generate area path for gradient fill
	const generateAreaPath = () => {
		const points = data.map((point, index) => {
			const x = (index / (data.length - 1)) * chartWidth;
			const y =
				chartHeight - ((point.value - minValue) / valueRange) * chartHeight;
			return `${x},${y}`;
		});

		const firstPoint = points[0];
		const lastPoint = points[points.length - 1];
		const lastX = lastPoint.split(",")[0];

		return `M ${firstPoint} L ${points.join(" L ")} L ${lastX},${chartHeight} L 0,${chartHeight} Z`;
	};

	// Calculate trend
	const firstValue = data[0]?.value || 0;
	const lastValue = data[data.length - 1]?.value || 0;
	const trend = lastValue > firstValue ? "up" : "down";
	const trendPercentage = firstValue !== 0
		? ((lastValue - firstValue) / Math.abs(firstValue)) * 100
		: lastValue !== 0
			? (lastValue > 0 ? 100 : -100)
			: 0;

	return (
		<Card className={className}>
			<CardHeader>
				<div className="flex items-center justify-between">
					<CardTitle className="text-base font-semibold">{title}</CardTitle>
					<div className="flex items-center space-x-1 text-sm">
						{trend === "up" ? (
							<TrendingUp className="h-4 w-4 text-green-500" />
						) : (
							<TrendingDown className="h-4 w-4 text-red-500" />
						)}
						<span
							className={cn(
								"font-medium",
								trend === "up" ? "text-green-600" : "text-red-600",
							)}
						>
							{Math.abs(trendPercentage).toFixed(1)}%
						</span>
					</div>
				</div>
				<div className="text-2xl font-bold text-gray-900 dark:text-white">
					{formatCurrencyCompact(lastValue)}
				</div>
			</CardHeader>

			<CardContent>
				<div className="relative" style={{ height }}>
					<svg
						width="100%"
						height={height}
						viewBox={`0 0 ${chartWidth + padding * 2} ${height}`}
						className="overflow-visible"
						role="img"
						aria-labelledby={titleId}
					>
						<title id={titleId}>
							{title} - Line chart showing financial data trends over time
						</title>
						{/* Gradient Definition */}
						<defs>
							<linearGradient id={gradientId} x1="0%" y1="0%" x2="0%" y2="100%">
								<stop offset="0%" stopColor={color} stopOpacity="0.3" />
								<stop offset="100%" stopColor={color} stopOpacity="0.05" />
							</linearGradient>
						</defs>

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

							{/* Vertical grid lines */}
							{data.map((point, index) => {
								if (index % Math.ceil(data.length / 5) === 0) {
									const x = (index / (data.length - 1)) * chartWidth;
									return (
										<line
											key={`v-grid-${point.date}-${index}`}
											x1={x}
											y1="0"
											x2={x}
											y2={chartHeight}
											stroke="#e5e7eb"
											strokeWidth="1"
											strokeDasharray="2,2"
											className="dark:stroke-gray-700"
										/>
									);
								}
								return null;
							})}
						</g>

						{/* Area Fill */}
						<g transform={`translate(${padding}, ${padding})`}>
							<path d={generateAreaPath()} fill={`url(#${gradientId})`} />
						</g>

						{/* Line */}
						<g transform={`translate(${padding}, ${padding})`}>
							<path
								d={generatePath()}
								fill="none"
								stroke={color}
								strokeWidth="2"
								strokeLinecap="round"
								strokeLinejoin="round"
							/>
						</g>

						{/* Data Points */}
						<g transform={`translate(${padding}, ${padding})`}>
							{data.map((point, index) => {
								const x = (index / (data.length - 1)) * chartWidth;
								const y =
									chartHeight -
									((point.value - minValue) / valueRange) * chartHeight;

								return (
									<circle
										key={`point-${point.date}`}
										cx={x}
										cy={y}
										r="3"
										fill={color}
										stroke="white"
										strokeWidth="2"
										className="transition-all cursor-pointer hover:stroke-[3]"
									>
										<title>{`${point.date}: ${formatCurrencyCompact(point.value)}`}</title>
									</circle>
								);
							})}
						</g>

						{/* Y-axis labels */}
						<g transform={`translate(${padding - 10}, ${padding})`}>
							{[0, 0.25, 0.5, 0.75, 1].map((ratio) => {
								const value = minValue + (maxValue - minValue) * (1 - ratio);
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
										{formatCurrencyCompact(value)}
									</text>
								);
							})}
						</g>

						{/* X-axis labels */}
						<g transform={`translate(${padding}, ${height - padding + 15})`}>
							{data.map((point, index) => {
								if (index % Math.ceil(data.length / 5) === 0) {
									const x = (index / (data.length - 1)) * chartWidth;
									const date = new Date(point.date);
									const label = date.toLocaleDateString("en-US", {
										month: "short",
										day: "numeric",
									});

									return (
										<text
											key={`x-label-${point.date}`}
											x={x}
											y="0"
											textAnchor="middle"
											className="text-xs fill-gray-500 dark:fill-gray-400"
										>
											{label}
										</text>
									);
								}
								return null;
							})}
						</g>
					</svg>
				</div>
			</CardContent>
		</Card>
	);
}
