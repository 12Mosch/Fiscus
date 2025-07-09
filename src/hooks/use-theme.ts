import {
	type Theme,
	useThemeStore as useThemeStoreOriginal,
} from "@/stores/theme-store";

/**
 * Hook to access theme state and actions directly from the Zustand store
 * This provides an alternative to the context-based useTheme from ThemeProvider
 * Use this when you need direct access to the store without context
 */
export function useThemeStore() {
	return useThemeStoreOriginal();
}

/**
 * Hook that provides theme utilities and computed values
 * This is a convenience hook that wraps the store with additional utilities
 */
export function useThemeUtils() {
	const { theme, resolvedTheme, systemTheme, setTheme } =
		useThemeStoreOriginal();

	const isDark = resolvedTheme === "dark";
	const isLight = resolvedTheme === "light";
	const isSystem = theme === "system";

	const toggleTheme = () => {
		setTheme(isDark ? "light" : "dark");
	};

	const setLightTheme = () => setTheme("light");
	const setDarkTheme = () => setTheme("dark");
	const setSystemTheme = () => setTheme("system");

	return {
		theme,
		resolvedTheme,
		systemTheme,
		setTheme,
		isDark,
		isLight,
		isSystem,
		toggleTheme,
		setLightTheme,
		setDarkTheme,
		setSystemTheme,
	};
}

export type { Theme };
