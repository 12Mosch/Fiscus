/**
 * Dashboard Route
 * Main dashboard route with layout and page components
 */

import { createFileRoute } from "@tanstack/react-router";
import { DashboardPage } from "@/components/dashboard/DashboardPage";
import { DashboardLayout } from "@/components/layout/DashboardLayout";

export const Route = createFileRoute("/")({
	component: Dashboard,
});

function Dashboard() {
	return (
		<DashboardLayout>
			<DashboardPage />
		</DashboardLayout>
	);
}
