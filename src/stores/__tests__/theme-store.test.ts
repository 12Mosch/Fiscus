import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock document.documentElement before importing the store
Object.defineProperty(document, "documentElement", {
	writable: true,
	value: {
		classList: {
			remove: vi.fn(),
			add: vi.fn(),
		},
	},
});

// Import after mocks are set up
import { useThemeStore, cleanupThemeStore } from "../theme-store";

beforeEach(() => {
	// Reset document.documentElement mocks
	vi.clearAllMocks();
});

afterEach(() => {
	// Clean up after each test
	cleanupThemeStore();
});

describe("Theme Store", () => {
	it("should initialize with system theme", () => {
		const store = useThemeStore.getState();
		expect(store.theme).toBe("system");
		expect(store.systemTheme).toBe("light"); // Default when matchMedia.matches is false
		expect(store.resolvedTheme).toBe("light");
	});

	it("should set theme correctly", () => {
		const store = useThemeStore.getState();
		store.setTheme("dark");

		expect(useThemeStore.getState().theme).toBe("dark");
		expect(useThemeStore.getState().resolvedTheme).toBe("dark");
	});

	it("should update system theme correctly", () => {
		const store = useThemeStore.getState();
		store.updateSystemTheme("dark");

		expect(useThemeStore.getState().systemTheme).toBe("dark");
		expect(useThemeStore.getState().resolvedTheme).toBe("dark"); // Should update since theme is "system"
	});

	it("should resolve theme correctly based on system preference", () => {
		const store = useThemeStore.getState();

		// Test system theme resolution
		store.setTheme("system");
		store.updateSystemTheme("dark");
		expect(useThemeStore.getState().resolvedTheme).toBe("dark");

		store.updateSystemTheme("light");
		expect(useThemeStore.getState().resolvedTheme).toBe("light");

		// Test explicit theme (should ignore system)
		store.setTheme("dark");
		store.updateSystemTheme("light");
		expect(useThemeStore.getState().resolvedTheme).toBe("dark");
	});

	it("should handle cleanup function without errors", () => {
		// Should not throw error when called
		expect(() => cleanupThemeStore()).not.toThrow();

		// Should handle multiple calls safely
		expect(() => cleanupThemeStore()).not.toThrow();
	});

	it("should handle cleanup when window is undefined", () => {
		// Mock window as undefined
		const originalWindow = global.window;
		// @ts-expect-error - Testing undefined window scenario
		delete global.window;

		// Should not throw error
		expect(() => cleanupThemeStore()).not.toThrow();

		// Restore window
		global.window = originalWindow;
	});
});
