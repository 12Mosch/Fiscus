import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Theme = "dark" | "light" | "system";

interface ThemeState {
	theme: Theme;
	resolvedTheme: "dark" | "light";
	systemTheme: "dark" | "light";
}

interface ThemeActions {
	setTheme: (theme: Theme) => void;
	updateSystemTheme: (systemTheme: "dark" | "light") => void;
	applyTheme: () => void;
}

export type ThemeStore = ThemeState & ThemeActions;

// Helper function to get system theme preference
const getSystemTheme = (): "dark" | "light" => {
	if (typeof window === "undefined") return "light";
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light";
};

// Helper function to resolve theme based on current theme and system preference
const resolveTheme = (
	theme: Theme,
	systemTheme: "dark" | "light",
): "dark" | "light" => {
	return theme === "system" ? systemTheme : theme;
};

// Helper function to apply theme to DOM
const applyThemeToDOM = (resolvedTheme: "dark" | "light") => {
	if (typeof window === "undefined") return;

	const root = window.document.documentElement;
	root.classList.remove("light", "dark");
	root.classList.add(resolvedTheme);
};

export const useThemeStore = create<ThemeStore>()(
	persist(
		(set, get) => ({
			// Initial state
			theme: "system",
			resolvedTheme: getSystemTheme(),
			systemTheme: getSystemTheme(),

			// Actions
			setTheme: (theme: Theme) => {
				const { systemTheme } = get();
				const resolvedTheme = resolveTheme(theme, systemTheme);

				set({ theme, resolvedTheme });
				applyThemeToDOM(resolvedTheme);
			},

			updateSystemTheme: (systemTheme: "dark" | "light") => {
				const { theme } = get();
				const resolvedTheme = resolveTheme(theme, systemTheme);

				set({ systemTheme, resolvedTheme });

				// Only apply to DOM if using system theme
				if (theme === "system") {
					applyThemeToDOM(resolvedTheme);
				}
			},

			applyTheme: () => {
				const { resolvedTheme } = get();
				applyThemeToDOM(resolvedTheme);
			},
		}),
		{
			name: "fiscus-theme-storage",
			partialize: (state) => ({ theme: state.theme }),
			onRehydrateStorage: () => (state) => {
				if (state) {
					// Update system theme and resolved theme after rehydration
					const systemTheme = getSystemTheme();
					const resolvedTheme = resolveTheme(state.theme, systemTheme);

					state.systemTheme = systemTheme;
					state.resolvedTheme = resolvedTheme;

					// Apply theme to DOM
					applyThemeToDOM(resolvedTheme);
				}
			},
		},
	),
);

// Store reference to media query and handler for cleanup
let mediaQuery: MediaQueryList | null = null;
let handleSystemThemeChange: ((e: MediaQueryListEvent) => void) | null = null;

// Export cleanup function for proper resource management
export const cleanupThemeStore = () => {
	if (typeof window !== "undefined" && mediaQuery && handleSystemThemeChange) {
		mediaQuery.removeEventListener("change", handleSystemThemeChange);
		mediaQuery = null;
		handleSystemThemeChange = null;
	}
};

// Initialize system theme listener
if (typeof window !== "undefined") {
	mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

	handleSystemThemeChange = (e: MediaQueryListEvent) => {
		const systemTheme = e.matches ? "dark" : "light";
		useThemeStore.getState().updateSystemTheme(systemTheme);
	};

	// Add listener for system theme changes
	mediaQuery.addEventListener("change", handleSystemThemeChange);
}
