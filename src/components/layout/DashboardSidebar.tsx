/**
 * Dashboard Sidebar Component
 * Navigation sidebar with collapsible functionality
 */

import { Link, useLocation } from "@tanstack/react-router";
import {
	BarChart3,
	CreditCard,
	LayoutDashboard,
	Menu,
	PieChart,
	Receipt,
	Settings,
	Target,
	X,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import { cn } from "@/lib/utils";
import type { NavigationItem } from "@/types/dashboard";

// Navigation items with icons
const navigationItems: NavigationItem[] = [
	{
		id: "dashboard",
		label: "Dashboard",
		href: "/dashboard",
		icon: <LayoutDashboard className="h-5 w-5" />,
	},
	{
		id: "accounts",
		label: "Accounts",
		href: "/accounts",
		icon: <CreditCard className="h-5 w-5" />,
	},
	{
		id: "transactions",
		label: "Transactions",
		href: "/transactions",
		icon: <Receipt className="h-5 w-5" />,
	},
	{
		id: "budgets",
		label: "Budgets",
		href: "/budgets",
		icon: <Target className="h-5 w-5" />,
		badge: "3",
	},
	{
		id: "goals",
		label: "Goals",
		href: "/goals",
		icon: <PieChart className="h-5 w-5" />,
	},
	{
		id: "reports",
		label: "Reports",
		href: "/reports",
		icon: <BarChart3 className="h-5 w-5" />,
	},
];

const bottomNavigationItems: NavigationItem[] = [
	{
		id: "settings",
		label: "Settings",
		href: "/settings",
		icon: <Settings className="h-5 w-5" />,
	},
];

interface DashboardSidebarProps {
	collapsed?: boolean;
	onToggle?: () => void;
}

export function DashboardSidebar({
	collapsed = false,
	onToggle,
}: DashboardSidebarProps) {
	const location = useLocation();
	const currentPath = location.pathname;

	const renderNavigationItem = (item: NavigationItem) => {
		const isActive =
			currentPath === item.href || currentPath.startsWith(`${item.href}/`);

		return (
			<Link
				key={item.id}
				to={item.href}
				className={cn(
					"flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
					"hover:bg-gray-100 dark:hover:bg-gray-800",
					isActive
						? "bg-blue-50 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300"
						: "text-gray-700 dark:text-gray-300",
					collapsed && "justify-center px-2",
				)}
				aria-label={collapsed ? item.label : undefined}
			>
				<span className="flex-shrink-0">{item.icon}</span>
				{!collapsed && (
					<>
						<span className="flex-1">{item.label}</span>
						{item.badge && (
							<Badge variant="secondary" className="ml-auto">
								{item.badge}
							</Badge>
						)}
					</>
				)}
			</Link>
		);
	};

	return (
		<>
			{/* Desktop Sidebar */}
			<aside
				className={cn(
					"fixed left-0 top-0 z-30 h-full bg-white dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 transition-all duration-300 ease-in-out",
					collapsed ? "w-16" : "w-64",
				)}
			>
				<div className="flex h-full flex-col">
					{/* Logo/Brand */}
					<div
						className={cn(
							"flex items-center border-b border-gray-200 dark:border-gray-800 px-4 py-4",
							collapsed && "justify-center px-2",
						)}
					>
						{collapsed ? (
							<div className="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600 text-white font-bold">
								F
							</div>
						) : (
							<div className="flex items-center gap-2">
								<div className="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600 text-white font-bold">
									F
								</div>
								<span className="text-xl font-bold text-gray-900 dark:text-white">
									Fiscus
								</span>
							</div>
						)}
					</div>

					{/* Navigation */}
					<nav className="flex-1 space-y-1 p-4">
						{navigationItems.map(renderNavigationItem)}
					</nav>

					{/* Bottom Navigation */}
					<div className="border-t border-gray-200 dark:border-gray-800 p-4">
						<div className="space-y-1">
							{bottomNavigationItems.map(renderNavigationItem)}
						</div>
					</div>

					{/* Toggle Button */}
					<div
						className={cn(
							"border-t border-gray-200 dark:border-gray-800 p-4",
							collapsed && "px-2",
						)}
					>
						<Button
							variant="ghost"
							size="sm"
							onClick={onToggle}
							className={cn("w-full", collapsed && "px-2")}
							aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
						>
							{collapsed ? (
								<Menu className="h-4 w-4" />
							) : (
								<>
									<X className="h-4 w-4 mr-2" />
									Collapse
								</>
							)}
						</Button>
					</div>
				</div>
			</aside>
		</>
	);
}
