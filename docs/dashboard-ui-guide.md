# Finance Dashboard UI Guide

## Overview

This document provides a comprehensive guide to the Finance Dashboard UI components created for the Fiscus application. The dashboard is built using React 18+, TypeScript, TanStack Router, Zustand, Vite, TailwindCSS, and shadcn/ui components.

## Architecture

### Tech Stack
- **Frontend Framework**: React 18+ with TypeScript
- **Routing**: TanStack Router
- **State Management**: Zustand (for UI state and theme management)
- **Build Tool**: Vite with HMR support
- **Styling**: TailwindCSS with shadcn/ui components
- **Theme Management**: Context-based theme provider with Zustand store
- **Icons**: Lucide React

### Project Structure
```
src/
├── components/
│   ├── dashboard/          # Dashboard-specific components
│   │   ├── DashboardPage.tsx
│   │   ├── FinancialCard.tsx
│   │   ├── AccountCard.tsx
│   │   ├── TransactionList.tsx
│   │   └── BudgetOverview.tsx
│   ├── layout/             # Layout components
│   │   ├── DashboardLayout.tsx
│   │   ├── DashboardSidebar.tsx
│   │   └── DashboardHeader.tsx
│   ├── charts/             # Data visualization components
│   │   ├── LineChart.tsx
│   │   ├── PieChart.tsx
│   │   └── BarChart.tsx
│   ├── theme-provider.tsx  # Theme context provider
│   ├── mode-toggle.tsx     # Theme toggle components
│   └── ui/                 # shadcn/ui components
├── stores/
│   └── theme-store.ts      # Zustand theme store
├── hooks/
│   └── use-theme.ts        # Theme-related hooks
├── types/
│   └── dashboard.ts        # TypeScript interfaces
├── data/
│   └── mockData.ts         # Mock data for development
└── routes/
    └── dashboard.tsx       # Dashboard route
```

## Theme Management

The application uses a comprehensive theme management system built with Zustand and React Context for consistent dark/light mode support.

### Architecture

#### Theme Store (`src/stores/theme-store.ts`)
- **Zustand Store**: Centralized theme state management
- **Persistence**: Automatic localStorage persistence with rehydration
- **System Theme Detection**: Monitors OS theme preference changes
- **Type Safety**: Full TypeScript support with proper type definitions

#### Theme Provider (`src/components/theme-provider.tsx`)
- **React Context**: Provides theme state to all components
- **System Theme Listener**: Automatically updates when OS theme changes
- **DOM Management**: Handles CSS class application to document root
- **Initialization**: Sets up theme on app startup

#### Theme Components
- **ModeToggle**: Dropdown with Light/Dark/System options
- **SimpleThemeToggle**: Simple toggle button for Light/Dark switching

## Currency Formatting

The dashboard uses shared currency formatting utilities to ensure consistency across all components.

### Available Formatters
```typescript
import {
  formatCurrency,
  formatCurrencyCompact,
  formatCurrencyAbbreviated,
  formatPercentage,
  formatRelativeDate,
  formatTransactionDate
} from '@/lib/formatters';

// Standard currency formatting (2 decimal places)
formatCurrency(1234.56, "USD"); // "$1,234.56"
formatCurrency(-1234.56, "USD", { handleNegative: true }); // "-$1,234.56"
formatCurrency(1234.56, "USD", { showPositiveSign: true }); // "+$1,234.56"

// Compact formatting (no decimal places)
formatCurrencyCompact(1234.56, "USD"); // "$1,235"

// Abbreviated formatting for large amounts
formatCurrencyAbbreviated(1500000, "USD"); // "$1.5M"
formatCurrencyAbbreviated(1500, "USD"); // "$1.5K"

// Percentage formatting
formatPercentage(12.345); // "12.3%"
formatPercentage(12.345, 2); // "12.35%"

// Date formatting
formatRelativeDate(new Date()); // "5m ago", "2h ago", "Yesterday"
formatTransactionDate(new Date()); // "Today", "Yesterday", "3 days ago"
```

### Usage Guidelines
- Use `formatCurrency` for precise financial displays (account balances, transaction amounts)
- Use `formatCurrencyCompact` for charts and summaries where space is limited
- Use `formatCurrencyAbbreviated` for large amounts in charts (K, M, B suffixes)
- All formatters default to USD but accept any currency code
- Consistent 2 decimal places for financial precision, 0 decimal places for charts

### Usage

#### Basic Theme Usage
```typescript
import { useTheme } from '@/components/theme-provider';

function MyComponent() {
  const { theme, resolvedTheme, setTheme } = useTheme();

  return (
    <div>
      <p>Current theme: {theme}</p>
      <p>Resolved theme: {resolvedTheme}</p>
      <button onClick={() => setTheme('dark')}>Dark Mode</button>
    </div>
  );
}
```

#### Using Theme Store Directly
```typescript
import { useThemeUtils } from '@/hooks/use-theme';

function MyComponent() {
  const { isDark, isLight, toggleTheme } = useThemeUtils();

  return (
    <button onClick={toggleTheme}>
      {isDark ? 'Switch to Light' : 'Switch to Dark'}
    </button>
  );
}
```

#### Adding Theme Toggle
```typescript
import { ModeToggle } from '@/components/mode-toggle';

function Header() {
  return (
    <header>
      <ModeToggle />
    </header>
  );
}
```

### Features
- **Three Theme Options**: Light, Dark, and System (follows OS preference)
- **Persistent Storage**: User preference saved in localStorage
- **System Theme Detection**: Automatically responds to OS theme changes
- **Smooth Transitions**: CSS transitions for seamless theme switching
- **Context-based State**: Consistent state management across components
- **Type-safe Implementation**: Full TypeScript support

## Components

### Layout Components

#### DashboardLayout
Main layout wrapper that provides the overall structure with sidebar, header, and content area.

**Features:**
- Responsive sidebar with collapse functionality
- Mobile-friendly overlay
- Consistent spacing and layout

#### DashboardSidebar
Navigation sidebar with collapsible functionality and configurable navigation items.

**Features:**
- Icon-based navigation
- Active state indicators
- Badge support for notifications
- Responsive collapse/expand
- Configurable navigation items through props
- Separate main and bottom navigation sections

**Props:**
- `collapsed?: boolean` - Controls sidebar collapsed state
- `onToggle?: () => void` - Callback for toggle button
- `navigationItems?: NavigationItem[]` - Custom main navigation items (defaults to standard dashboard items)
- `bottomNavigationItems?: NavigationItem[]` - Custom bottom navigation items (defaults to settings)

#### DashboardHeader
Top navigation bar with user menu, notifications, and search.

**Features:**
- Global search functionality
- Notification center
- User profile dropdown
- Theme toggle with Light/Dark/System options
- Mobile-responsive design

### Dashboard Components

#### FinancialCard
Reusable card component for displaying financial metrics.

**Props:**
- `title`: Card title
- `value`: Main value (string or number)
- `change`: Optional change indicator with percentage and trend
- `icon`: Optional icon component
- `className`: Additional CSS classes

#### AccountCard
Displays individual account information with balance and details.

**Features:**
- Account type indicators with colors
- Balance formatting
- Last updated timestamps
- Account-specific information (credit limits, etc.)

#### TransactionList
Displays a list of recent transactions with filtering and formatting.

**Features:**
- Transaction type icons and colors
- Category badges
- Status indicators
- Merchant information
- Responsive design

#### BudgetOverview
Shows budget progress with visual indicators and spending alerts.

**Features:**
- Progress bars for each budget category
- Status indicators (good, warning, exceeded)
- Remaining budget calculations
- Quick action buttons

### Chart Components

#### LineChart
SVG-based line chart for displaying trends over time.

**Features:**
- Responsive design
- Interactive data points
- Grid lines and axis labels
- Gradient fill areas
- Trend indicators

#### PieChart
SVG-based pie chart for category breakdowns.

**Features:**
- Interactive slices with tooltips
- Legend with percentages
- Summary statistics
- Responsive layout

#### BarChart
SVG-based bar chart for comparing values.

**Features:**
- Horizontal and vertical orientations
- Value labels
- Grid lines
- Summary statistics

## Data Structure

### TypeScript Interfaces

The dashboard uses comprehensive TypeScript interfaces defined in `src/types/dashboard.ts`:

- `Account`: Bank account information (includes optional `creditLimit` for credit accounts)
- `Transaction`: Financial transaction data
- `Budget`: Budget allocation and spending
- `SpendingCategory`: Categorized spending data
- `FinancialGoal`: Savings and financial goals
- `DashboardStats`: Overall financial statistics

### Mock Data

Mock data is provided in `src/data/mockData.ts` for development and testing:

- Sample accounts (checking, savings, credit, investment)
- Transaction history with various categories
- Budget allocations and spending
- Chart data for visualizations

## Responsive Design

The dashboard is fully responsive and works on:

- **Desktop**: Full layout with sidebar and all components
- **Tablet**: Collapsible sidebar with optimized spacing
- **Mobile**: Mobile-first design with overlay navigation

### Breakpoints
- `sm`: 640px and up
- `md`: 768px and up
- `lg`: 1024px and up
- `xl`: 1280px and up

## Accessibility Features

The dashboard includes comprehensive accessibility features:

- **ARIA Labels**: Proper labeling for screen readers
- **Keyboard Navigation**: Full keyboard support
- **Color Contrast**: WCAG compliant color schemes
- **Focus Management**: Visible focus indicators
- **Semantic HTML**: Proper heading hierarchy and structure

## Styling

### TailwindCSS Classes
The dashboard uses utility-first CSS with TailwindCSS:

- Consistent spacing with the spacing scale
- Responsive design utilities
- Dark mode support
- Component variants

### shadcn/ui Integration
Built on top of shadcn/ui components:

- Card, Button, Badge, Avatar
- Dropdown menus and sheets
- Progress bars and separators
- Consistent design system

### Global CSS Override Notes
The project has a global CSS rule in `src/styles.css` that centers all h1 elements:
```css
h1 {
    text-align: center;
}
```

To ensure proper left alignment of headings in dashboard components, use the `!text-left` TailwindCSS class to override this global rule:
```tsx
<h1 className="text-2xl font-bold text-gray-900 dark:text-white !text-left">
    Dashboard
</h1>
```

### Search Component Spacing
The search component in the DashboardHeader uses proper icon positioning to prevent text overlap:
```tsx
<div className="relative">
    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400 z-10 pointer-events-none" />
    <Input
        placeholder="Search transactions, accounts..."
        className="w-64 pl-10"
    />
</div>
```
- Icon positioned at `left-3` (12px from left)
- Input padding `pl-10` (40px) provides adequate clearance for icon + spacing

## Usage

### Basic Implementation

```tsx
import { DashboardLayout } from '@/components/layout/DashboardLayout';
import { DashboardPage } from '@/components/dashboard/DashboardPage';

function Dashboard() {
  return (
    <DashboardLayout>
      <DashboardPage />
    </DashboardLayout>
  );
}
```

### Custom Financial Card

```tsx
import { FinancialCard } from '@/components/dashboard/FinancialCard';
import { DollarSign } from 'lucide-react';

<FinancialCard
  title="Total Balance"
  value={62270.00}
  change={{
    value: 2.5,
    type: 'increase',
    period: 'last month'
  }}
  icon={<DollarSign className="h-5 w-5" />}
/>
```

### Credit Account Handling

The `AccountCard` component properly handles credit accounts with the following features:

- **Available Credit Calculation**: For credit accounts with a `creditLimit`, available credit is calculated as `creditLimit - currentDebt`
- **Balance Display**: Negative balances (debt) are shown in red, positive balances in green
- **Credit Information**: Shows both available credit and total credit limit
- **Fallback**: If no `creditLimit` is provided, the credit-specific section is hidden

```tsx
// Example credit account data
const creditAccount = {
  id: "acc-3",
  name: "Credit Card",
  type: "credit",
  balance: -1250.75, // Negative indicates debt
  currency: "USD",
  creditLimit: 5000, // Required for available credit calculation
  lastUpdated: new Date(),
  accountNumber: "****9012"
};
```

**Available Credit Calculation:**
- Current debt: $1,250.75 (absolute value of negative balance)
- Credit limit: $5,000.00
- Available credit: $3,749.25 ($5,000 - $1,250.75)

## Development

### Running the Dashboard

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Run linter
npm run lint
```

### Adding New Components

1. Create component in appropriate directory
2. Add TypeScript interfaces to `types/dashboard.ts`
3. Update mock data if needed
4. Add to main dashboard page
5. Test responsive design

## Future Enhancements

Potential improvements for the dashboard:

1. **Real Data Integration**: Replace mock data with API calls
2. **Advanced Charts**: Add more chart types and interactions
3. **Customization**: User-configurable dashboard layouts
4. **Animations**: Smooth transitions and micro-interactions
5. **Export Features**: PDF/CSV export functionality
6. **Filters**: Advanced filtering and date range selection

## Browser Support

The dashboard supports modern browsers:

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Performance

The dashboard is optimized for performance:

- **Code Splitting**: Route-based code splitting with TanStack Router
- **Lazy Loading**: Components loaded on demand
- **Memoization**: React.memo for expensive components
- **Efficient Rendering**: Minimal re-renders with proper key usage
