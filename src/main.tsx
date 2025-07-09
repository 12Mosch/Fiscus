import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { ThemeProvider } from "@/components/theme-provider";
import { routeTree } from "./routeTree.gen";

// Create a new router instance
const router = createRouter({ routeTree });

// Register the router instance for type safety
declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

// Render the app
const rootElement = document.getElementById("root");
if (rootElement && !rootElement.hasChildNodes()) {
	const root = ReactDOM.createRoot(rootElement);
	root.render(
		<StrictMode>
			<ThemeProvider defaultTheme="system" storageKey="fiscus-theme-storage">
				<RouterProvider router={router} />
			</ThemeProvider>
		</StrictMode>,
	);
}
