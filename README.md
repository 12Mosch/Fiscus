# Fiscus - Tauri + React

A modern desktop application built with Tauri 2.0 and React, providing a secure and performant cross-platform experience.

## Tech Stack

- **Frontend**: React 18+ with TypeScript
- **Build Tool**: Vite with HMR support
- **Backend**: Rust with Tauri 2.0
- **Styling**: CSS with TailwindCSS integration
- **Development**: ESLint for code quality

## Features

- ⚡ Fast development with Vite HMR
- 🔒 Secure Tauri 2.0 architecture
- 🎨 Modern React components with TypeScript
- 🌙 Dark mode support
- 📱 Responsive design
- 🔧 Comprehensive Tauri API integration

## Getting Started

### Prerequisites

- Node.js (v18 or higher)
- Rust (latest stable)
- Platform-specific dependencies for Tauri

### Installation

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```

### Development

```bash
# Start development server (web only)
npm run dev

# Start Tauri development mode
npm run tauri dev

# Run linter
npm run lint
```

### Building

```bash
# Build for production
npm run tauri build
```

## Project Structure

```
src/
├── components/          # React components
├── assets/             # Static assets
├── App.tsx             # Main React application
├── main.tsx            # React entry point
└── styles.css          # Global styles

src-tauri/              # Rust backend
docs/                   # Documentation
├── react-integration.md # React setup guide
```

## Documentation

- [React Integration Guide](docs/react-integration.md) - Detailed setup and usage guide
- [Tauri Documentation](https://v2.tauri.app/) - Official Tauri docs

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [ES7+ React/Redux/React-Native snippets](https://marketplace.visualstudio.com/items?itemName=dsznajder.es7-react-js-snippets)
