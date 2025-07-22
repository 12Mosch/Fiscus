# Fiscus

A modern desktop application built with Tauri 2.0 and React, providing a secure and performant cross-platform experience.

## Tech Stack

- **Frontend**: React 19+ with TypeScript
- **Routing**: TanStack Router
- **State Management**:
  - **Backend**: TanStack Router loader-function
  - **UI / Global**: Zustand
- **Build Tool**: Vite with HMR support
- **Backend**: Rust with Tauri 2.0
- **Database**: SQLite with Tauri SQL plugin
- **Styling**: CSS with TailwindCSS and shadcn/ui integration
- **Development**: Biome for code quality

## Features

- ⚡ Fast development with Vite HMR
- 🔒 Secure Tauri 2.0 architecture
- 🎨 Modern React components with TypeScript
- 🌙 Dark mode support
- 📱 Responsive design
- 🔧 Comprehensive Tauri API integration
- 🗄️ SQLite database with type-safe operations
- 📊 Personal finance management capabilities

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

## 🔒 Security Architecture

Fiscus implements a secure architecture with multiple layers of protection:

### Data Security

- **Encryption**: All sensitive financial data is encrypted using AES-256-GCM
- **Key Management**: Secure key derivation and rotation using PBKDF2/Argon2
- **Storage**: No plaintext financial data stored locally

### API Security

- **Secure Commands**: All database operations go through encrypted Tauri commands
- **Authentication**: User authentication and session management
- **Authorization**: Ownership verification for all data access
- **Input Validation**: Comprehensive validation and SQL injection protection

### Development Guidelines

⚠️ **Critical**: Never use direct database access in frontend code. Always use the secure API service:

```typescript
// ✅ Correct - Use secure API service
import { apiService } from '@/lib/api-service';
const accounts = await apiService.accounts.findByUserId(userId);

// ❌ Wrong - Direct database access (deprecated and removed)
// The old database service has been removed for security reasons.
// Use the secure API service instead:
```

For migration guidance, see [Security Migration Guide](docs/security-migration-guide.md).

## Project Structure

```text
src/
├── components/          # React components
├── assets/             # Static assets
├── lib/                # Library code
│   ├── api-service/    # Secure API service
│   └── database/       # Database hooks
├── App.tsx             # Main React application
├── main.tsx            # React entry point
└── styles.css          # Global styles

src-tauri/              # Rust backend
├── src/
│   ├── commands/       # Domain-specific command modules
│   ├── database/       # Database utilities
│   ├── encryption/     # Encryption services
│   └── ...
docs/                   # Documentation
├── react-integration.md # React setup guide
├── api-reference.md    # API documentation
├── security-migration-guide.md # Migration guide
└── ...
```

## Documentation

### API Documentation

- [API Reference](docs/api-reference.md) - Complete API documentation with examples
- [API Testing Guide](docs/api-testing-guide.md) - Testing strategies for Rust commands and TypeScript client
- [API Security Guide](docs/api-security-guide.md) - Security best practices and implementation

### Development Guides

- [React Integration Guide](docs/react-integration.md) - Detailed setup and usage guide
- [Database Integration](docs/database-integration.md) - Database setup, schema, and usage patterns
- [Dashboard UI Guide](docs/dashboard-ui-guide.md) - UI component documentation
- [Development Tools](docs/development-tools.md) - Development workflow and tools

### External Resources

- [Tauri Documentation](https://v2.tauri.app/) - Official Tauri docs

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [ES7+ React/Redux/React-Native snippets](https://marketplace.visualstudio.com/items?itemName=dsznajder.es7-react-js-snippets)
