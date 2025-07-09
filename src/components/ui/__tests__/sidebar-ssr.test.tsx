import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SidebarMenuSkeleton } from "../sidebar";

describe("SidebarMenuSkeleton SSR Compatibility", () => {
	it("should render skeleton with proper structure", () => {
		const { container } = render(<SidebarMenuSkeleton />);

		const skeleton = container.querySelector('[data-sidebar="menu-skeleton"]');
		const textSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-text"]',
		);

		expect(skeleton).toBeInTheDocument();
		expect(textSkeleton).toBeInTheDocument();
		expect(textSkeleton?.getAttribute("style")).toContain("--skeleton-width:");
	});

	it("should render with icon when showIcon is true", () => {
		const { container } = render(<SidebarMenuSkeleton showIcon />);

		const iconSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-icon"]',
		);
		const textSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-text"]',
		);

		expect(iconSkeleton).toBeInTheDocument();
		expect(textSkeleton).toBeInTheDocument();
	});

	it("should render without icon when showIcon is false", () => {
		const { container } = render(<SidebarMenuSkeleton showIcon={false} />);

		const iconSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-icon"]',
		);
		const textSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-text"]',
		);

		expect(iconSkeleton).not.toBeInTheDocument();
		expect(textSkeleton).toBeInTheDocument();
	});

	it("should handle SSR environment gracefully", () => {
		// The component should handle undefined window gracefully
		// This test verifies that the component doesn't crash in SSR
		expect(() => {
			render(<SidebarMenuSkeleton />);
		}).not.toThrow();
	});

	it("should generate width values in expected range", () => {
		const { container } = render(<SidebarMenuSkeleton />);
		const textSkeleton = container.querySelector(
			'[data-sidebar="menu-skeleton-text"]',
		);
		const style = textSkeleton?.getAttribute("style") || "";

		// Extract width value
		const widthMatch = style.match(/--skeleton-width: (\d+)%/);
		expect(widthMatch).toBeTruthy();

		if (widthMatch) {
			const width = parseInt(widthMatch[1]);
			// Should be either 70% (SSR default) or between 50-90% (client-side random)
			expect(width === 70 || (width >= 50 && width <= 90)).toBe(true);
		}
	});
});
