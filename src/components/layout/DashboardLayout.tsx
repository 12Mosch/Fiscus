/**
 * Main Dashboard Layout Component
 * Provides the overall structure with sidebar, header, and content area
 */

import { useState } from "react";
import { cn } from "@/lib/utils";
import type { DashboardLayoutProps } from "@/types/dashboard";
import { DashboardHeader } from "./DashboardHeader";
import { DashboardSidebar } from "./DashboardSidebar";

export function DashboardLayout({ children }: DashboardLayoutProps) {
	const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

	const toggleSidebar = () => {
		setSidebarCollapsed(!sidebarCollapsed);
	};

	return (
		<div className="min-h-screen bg-gray-50 dark:bg-gray-900">
			{/* Sidebar */}
			<DashboardSidebar collapsed={sidebarCollapsed} onToggle={toggleSidebar} />

			{/* Main Content Area */}
			<div
				className={cn(
					"transition-all duration-300 ease-in-out",
					sidebarCollapsed ? "ml-16" : "ml-64",
				)}
			>
				{/* Header */}
				<DashboardHeader onMenuClick={toggleSidebar} />

				{/* Page Content */}
				<main className="p-6">
					<div className="mx-auto max-w-7xl">{children}</div>
				</main>
			</div>

			{/* Mobile Overlay */}
			{!sidebarCollapsed && (
				<div
					className="fixed inset-0 z-20 bg-black bg-opacity-50 lg:hidden"
					onClick={toggleSidebar}
					aria-hidden="true"
				/>
			)}
		</div>
	);
}
