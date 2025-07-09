/**
 * Tests for DashboardSidebar Component
 */

import { render, screen } from "@testing-library/react";
import { Home, User } from "lucide-react";
import { describe, expect, it, vi } from "vitest";
import type { NavigationItem } from "@/types/dashboard";
import { DashboardSidebar } from "../DashboardSidebar";

// Mock the router
vi.mock("@tanstack/react-router", async (importOriginal) => {
	const actual = (await importOriginal()) as Record<string, unknown>;
	return {
		...actual,
		useLocation: () => ({ pathname: "/dashboard" }),
		Link: ({
			children,
			to,
			className,
			...props
		}: {
			children: React.ReactNode;
			to: string;
			className?: string;
			[key: string]: unknown;
		}) => (
			<a href={to} className={className} {...props}>
				{children}
			</a>
		),
	};
});

// Simple wrapper component to replace MemoryRouter for testing
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
	<div>{children}</div>
);

const customNavigationItems: NavigationItem[] = [
	{
		id: "home",
		label: "Home",
		href: "/home",
		icon: <Home className="h-5 w-5" />,
	},
	{
		id: "profile",
		label: "Profile",
		href: "/profile",
		icon: <User className="h-5 w-5" />,
		badge: "New",
	},
];

const customBottomNavigationItems: NavigationItem[] = [
	{
		id: "help",
		label: "Help",
		href: "/help",
		icon: <User className="h-5 w-5" />,
	},
];

describe("DashboardSidebar", () => {
	it("renders with default navigation items", () => {
		render(
			<TestWrapper>
				<DashboardSidebar />
			</TestWrapper>,
		);

		// Check for default navigation items
		expect(screen.getByText("Dashboard")).toBeInTheDocument();
		expect(screen.getByText("Accounts")).toBeInTheDocument();
		expect(screen.getByText("Transactions")).toBeInTheDocument();
		expect(screen.getByText("Budgets")).toBeInTheDocument();
		expect(screen.getByText("Goals")).toBeInTheDocument();
		expect(screen.getByText("Reports")).toBeInTheDocument();
		expect(screen.getByText("Settings")).toBeInTheDocument();
	});

	it("renders with custom navigation items", () => {
		render(
			<TestWrapper>
				<DashboardSidebar
					navigationItems={customNavigationItems}
					bottomNavigationItems={customBottomNavigationItems}
				/>
			</TestWrapper>,
		);

		// Check for custom navigation items
		expect(screen.getByText("Home")).toBeInTheDocument();
		expect(screen.getByText("Profile")).toBeInTheDocument();
		expect(screen.getByText("Help")).toBeInTheDocument();

		// Check that default items are not present
		expect(screen.queryByText("Accounts")).not.toBeInTheDocument();
		expect(screen.queryByText("Transactions")).not.toBeInTheDocument();
		expect(screen.queryByText("Settings")).not.toBeInTheDocument();
	});

	it("renders badges when provided", () => {
		render(
			<TestWrapper>
				<DashboardSidebar navigationItems={customNavigationItems} />
			</TestWrapper>,
		);

		expect(screen.getByText("New")).toBeInTheDocument();
	});

	it("renders collapsed state correctly", () => {
		render(
			<TestWrapper>
				<DashboardSidebar collapsed={true} />
			</TestWrapper>,
		);

		// In collapsed state, the brand should show only "F"
		const brandElement = screen.getByText("F");
		expect(brandElement).toBeInTheDocument();

		// The full "Fiscus" text should not be visible
		expect(screen.queryByText("Fiscus")).not.toBeInTheDocument();
	});

	it("calls onToggle when toggle button is clicked", () => {
		const mockOnToggle = vi.fn();
		render(
			<TestWrapper>
				<DashboardSidebar onToggle={mockOnToggle} />
			</TestWrapper>,
		);

		const toggleButton = screen.getByLabelText("Collapse sidebar");
		toggleButton.click();

		expect(mockOnToggle).toHaveBeenCalledTimes(1);
	});
});
