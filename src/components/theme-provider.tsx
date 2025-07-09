import { createContext, type ReactNode, useContext, useEffect } from "react";
import { type Theme, useThemeStore } from "@/stores/theme-store";

type ThemeProviderProps = {
	children: ReactNode;
	defaultTheme?: Theme;
	storageKey?: string;
};

type ThemeProviderState = {
	theme: Theme;
	resolvedTheme: "dark" | "light";
	systemTheme: "dark" | "light";
	setTheme: (theme: Theme) => void;
};

const initialState: ThemeProviderState = {
	theme: "system",
	resolvedTheme: "light",
	systemTheme: "light",
	setTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

export function ThemeProvider({
	children,
	defaultTheme = "system",
	storageKey = "fiscus-theme-storage",
	...props
}: ThemeProviderProps) {
	const {
		theme,
		resolvedTheme,
		systemTheme,
		setTheme,
		updateSystemTheme,
		applyTheme,
	} = useThemeStore();

	useEffect(() => {
		// Set default theme if no theme is stored
		if (!theme) {
			setTheme(defaultTheme);
		}
	}, [defaultTheme, setTheme, theme]);

	useEffect(() => {
		// Apply theme on mount and when resolved theme changes
		applyTheme();
	}, [applyTheme]);

	useEffect(() => {
		// Listen for system theme changes
		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

		const handleSystemThemeChange = (e: MediaQueryListEvent) => {
			const newSystemTheme = e.matches ? "dark" : "light";
			updateSystemTheme(newSystemTheme);
		};

		// Add listener
		mediaQuery.addEventListener("change", handleSystemThemeChange);

		// Cleanup listener on unmount
		return () => {
			mediaQuery.removeEventListener("change", handleSystemThemeChange);
		};
	}, [updateSystemTheme]);

	const value: ThemeProviderState = {
		theme,
		resolvedTheme,
		systemTheme,
		setTheme,
	};

	return (
		<ThemeProviderContext.Provider {...props} value={value}>
			{children}
		</ThemeProviderContext.Provider>
	);
}

export const useTheme = () => {
	const context = useContext(ThemeProviderContext);

	if (context === undefined) {
		throw new Error("useTheme must be used within a ThemeProvider");
	}

	return context;
};
