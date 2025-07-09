# Finance Dashboard UI Guide

## Overview

This document provides a comprehensive guide to the Finance Dashboard UI components created for the Fiscus application. The dashboard is built using React 18+, TypeScript, TanStack Router, Zustand, Vite, TailwindCSS, and shadcn/ui components.

## Architecture

### Tech Stack
- **Frontend Framework**: React 18+ with TypeScript
- **Routing**: TanStack Router
- **State Management**: Zustand (for UI state)
- **Build Tool**: Vite with HMR support
- **Styling**: TailwindCSS with shadcn/ui components
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
│   └── ui/                 # shadcn/ui components
├── types/
│   └── dashboard.ts        # TypeScript interfaces
├── data/
│   └── mockData.ts         # Mock data for development
└── routes/
    └── dashboard.tsx       # Dashboard route
```

## Components

### Layout Components

#### DashboardLayout
Main layout wrapper that provides the overall structure with sidebar, header, and content area.

**Features:**
- Responsive sidebar with collapse functionality
- Mobile-friendly overlay
- Consistent spacing and layout

#### DashboardSidebar
Navigation sidebar with collapsible functionality.

**Features:**
- Icon-based navigation
- Active state indicators
- Badge support for notifications
- Responsive collapse/expand

#### DashboardHeader
Top navigation bar with user menu, notifications, and search.

**Features:**
- Global search functionality
- Notification center
- User profile dropdown
- Dark mode toggle
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

- `Account`: Bank account information
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
