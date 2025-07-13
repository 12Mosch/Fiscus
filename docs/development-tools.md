# Development Tools

This document outlines the development tools and debug components available in the Fiscus application.

## Debug Components

### DatabaseTest Component

The `DatabaseTest` component provides a comprehensive testing interface for database operations in the Tauri environment.

#### Features

- **Connection Testing**: Verifies database connectivity and version
- **Account Operations**: Tests account creation and management
- **Transaction Operations**: Tests transaction creation and balance updates
- **Real-time Results**: Displays test results with timestamps
- **Error Handling**: Comprehensive error reporting and logging

#### Production Safety

The `DatabaseTest` component implements build-time conditional rendering to ensure it's never included in production builds:

```typescript
// Conditional export based on environment
export const DatabaseTest = process.env.NODE_ENV !== 'production' 
  ? DatabaseTestImpl 
  : function ProductionStub() {
      console.warn('DatabaseTest component is not available in production builds');
      return null;
    };
```

#### Environment Behavior

- **Development Mode** (`NODE_ENV !== 'production'`): Full component functionality available
- **Production Mode** (`NODE_ENV === 'production'`): Component returns `null` and logs a warning
- **Test Environment**: Full component functionality available for testing

#### Usage

```typescript
import { DatabaseTest } from '@/components/debug/DatabaseTest';

function DevelopmentPage() {
  return (
    <div>
      <h1>Development Tools</h1>
      <DatabaseTest />
    </div>
  );
}
```

#### Testing

The component includes comprehensive tests to verify conditional rendering:

```bash
# Run DatabaseTest component tests
npm test -- src/components/debug/__tests__/DatabaseTest.test.tsx
```

Test coverage includes:

- Development mode rendering
- Production mode stub behavior
- Warning message logging
- Environment variable handling

### DatabaseSeeder Component (Removed)

The `DatabaseSeeder` component has been **removed** as part of the security migration. Database seeding functionality is no longer available through the UI.

#### Migration to Secure API Service

For development data creation, use the secure API service programmatically:

```typescript
import { apiService } from '@/lib/api-service';

// Create test data using the secure API
const account = await apiService.accounts.create(accountData);
const transaction = await apiService.transactions.create(transactionData);
```

## Development Routes

### `/dev` Route

The development route (`src/routes/dev.tsx`) provides access to all development tools:

- Database testing utilities
- Database seeding interface
- Development environment information
- Debug component showcase

#### Access Control

The development route is available in all environments but debug components within it implement their own production safety measures.

## Best Practices

### Component Development

1. **Environment Checks**: Always implement environment-based conditional rendering for debug components
2. **Production Safety**: Ensure debug components are excluded from production builds
3. **Warning Messages**: Log appropriate warnings when debug components are accessed in production
4. **Testing**: Include tests for both development and production behavior

### Build-time Optimization

Debug components use conditional exports that allow bundlers to tree-shake unused code in production builds:

```typescript
// This pattern allows tree-shaking in production
export const DebugComponent = process.env.NODE_ENV !== 'production' 
  ? ActualComponent 
  : () => null;
```

### Security Considerations

- Debug components should never expose sensitive data in production
- Database operations in debug components should be limited to test data
- Always validate environment before performing destructive operations

## Available Scripts

```bash
# Run all tests including debug component tests
npm run test

# Run linter (includes debug components)
npm run lint

# Start development server with debug tools
npm run dev

# Build for production (excludes debug components)
npm run build
```

## Environment Variables

- `NODE_ENV`: Controls component availability and behavior
  - `development`: All debug components available
  - `production`: Debug components return null/stub
  - `test`: All debug components available for testing

## Troubleshooting

### Common Issues

1. **Debug Component Not Rendering**: Check `NODE_ENV` value
2. **Production Warnings**: Verify conditional exports are working correctly
3. **Test Failures**: Ensure environment variables are properly mocked in tests

### Debug Mode

Enable detailed logging for development tools:

```bash
DEBUG=development:* npm run dev
```
