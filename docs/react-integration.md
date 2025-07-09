# React Integration with Tauri 2.0

This document outlines the React integration setup for the Fiscus Tauri application, following current best practices for Tauri 2.0.

## Overview

The project has been successfully integrated with React 18+ using Vite as the build tool, providing:
- Hot Module Replacement (HMR) for fast development
- TypeScript support with JSX
- Tauri API integration with React components
- Modern React patterns and hooks

## Project Structure

```
src/
├── components/          # React components
│   └── TauriDemo.tsx   # Demo component showcasing Tauri APIs
├── assets/             # Static assets
│   ├── react.svg       # React logo
│   ├── vite.svg        # Vite logo
│   └── tauri.svg       # Tauri logo
├── App.tsx             # Main React application component
├── main.tsx            # React application entry point
└── styles.css          # Global styles with React-specific additions
```

## Key Dependencies

### Production Dependencies
- `react` - React library
- `react-dom` - React DOM rendering
- `@tauri-apps/api` - Tauri API bindings
- `@tauri-apps/plugin-opener` - Tauri opener plugin

### Development Dependencies
- `@types/react` - React TypeScript definitions
- `@types/react-dom` - React DOM TypeScript definitions
- `@vitejs/plugin-react` - Vite React plugin

## Configuration Files

### Vite Configuration (`vite.config.ts`)
```typescript
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

### TypeScript Configuration (`tsconfig.json`)
Key additions for React support:
```json
{
  "compilerOptions": {
    "jsx": "react-jsx"
  }
}
```

## React Components

### Main App Component (`src/App.tsx`)
The main application component that renders the logo section and integrates the TauriDemo component.

### TauriDemo Component (`src/components/TauriDemo.tsx`)
A comprehensive demo component showcasing:
- Rust command invocation using `invoke()`
- External URL opening using Tauri's opener plugin
- React state management with hooks
- Error handling and loading states
- Proper TypeScript typing

## Development Workflow

### Starting Development Server
```bash
# Start Vite development server only
npm run dev

# Start Tauri development mode (includes Vite server)
npm run tauri dev
```

### Code Quality
```bash
# Run ESLint
npm run lint

# Run tests (when available)
npm run test
```

### Building for Production
```bash
# Build the application
npm run tauri build
```

## Best Practices Implemented

1. **Component Structure**: Modular component architecture with clear separation of concerns
2. **TypeScript Integration**: Full TypeScript support with proper typing for Tauri APIs
3. **Error Handling**: Comprehensive error handling in async operations
4. **Accessibility**: Proper ARIA attributes and semantic HTML
5. **Performance**: React.StrictMode enabled for development checks
6. **Security**: `rel="noopener noreferrer"` on external links

## Tauri API Integration

### Command Invocation
```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke("greet", { name: "World" });
```

### External URL Opening
```typescript
import { open } from "@tauri-apps/plugin-opener";

await open("https://tauri.app");
```

## Styling

The project uses a combination of:
- Global CSS with CSS custom properties
- Component-specific styles
- Dark mode support
- Responsive design principles

## State Management

### Theme Store (`src/stores/theme-store.ts`)

The application uses Zustand for theme management with the following features:
- System theme detection and automatic updates
- Persistent theme preferences
- Dark/light/system theme modes
- Proper cleanup to prevent memory leaks

#### Usage
```typescript
import { useThemeStore, cleanupThemeStore } from "@/stores/theme-store";

// In a component
const { theme, setTheme, resolvedTheme } = useThemeStore();

// Cleanup when needed (e.g., in app teardown)
cleanupThemeStore();
```

#### Memory Management
The theme store automatically listens to system theme changes via `matchMedia`. To prevent memory leaks in applications that dynamically create/destroy the store, call `cleanupThemeStore()` during application teardown or component unmounting.

## Next Steps

1. Add unit tests using React Testing Library
2. Implement additional Tauri plugins as needed
3. Add more complex React components
4. Consider additional state management solutions for larger applications
5. Implement React Router for multi-page applications

## Troubleshooting

### Common Issues
1. **Build Errors**: Ensure all dependencies are installed with `npm install`
2. **TypeScript Errors**: Check that `jsx: "react-jsx"` is set in `tsconfig.json`
3. **Tauri API Issues**: Verify that Tauri commands are properly defined in Rust backend

### Development Tips
- Use React Developer Tools browser extension for debugging
- Enable React.StrictMode to catch potential issues early
- Utilize Vite's HMR for fast development iteration
