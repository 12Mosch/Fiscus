# Custom Hooks Documentation

This document describes the custom hooks created to extract calculations from the Dashboard components, improving testability and maintainability.

## Overview

The following custom hooks have been created to replace inline calculations in the `DashboardPage` component:

- `useDashboardMetrics` - Financial metrics calculations
- `useBudgetSummary` - Budget categorization and statistics
- `useChartData` - Chart data filtering and processing

## Hooks

### useDashboardMetrics

**Location**: `src/hooks/use-dashboard-metrics.ts`

**Purpose**: Calculates key financial metrics from dashboard statistics.

**Parameters**:
- `stats: DashboardStats` - Dashboard statistics object

**Returns**: `DashboardMetrics` object containing:
- `netWorth: number` - Total balance (same as totalBalance)
- `monthlyNet: number` - Monthly income minus expenses
- `expenseRatio: number` - Expense ratio as percentage (0-100)
- `savingsAmount: number` - Calculated savings amount based on savings rate
- `expenseRatioFormatted: string` - Formatted expense ratio with % symbol

**Example Usage**:
```typescript
const stats = mockDashboardStats;
const metrics = useDashboardMetrics(stats);

console.log(metrics.monthlyNet); // 1500
console.log(metrics.expenseRatioFormatted); // "70.0%"
```

### useBudgetSummary

**Location**: `src/hooks/use-budget-summary.ts`

**Purpose**: Categorizes budgets and calculates summary statistics.

**Parameters**:
- `budgets: Budget[]` - Array of budget objects

**Returns**: `BudgetSummary` object containing:
- `onTrack: number` - Count of budgets with <90% utilization
- `warning: number` - Count of budgets with 90-100% utilization
- `overLimit: number` - Count of budgets with >100% utilization
- `totalBudgets: number` - Total number of budgets
- `averageUtilization: number` - Average utilization percentage
- `totalAllocated: number` - Sum of all allocated amounts
- `totalSpent: number` - Sum of all spent amounts

**Example Usage**:
```typescript
const budgets = mockBudgets;
const summary = useBudgetSummary(budgets);

console.log(`${summary.onTrack} on track, ${summary.overLimit} over limit`);
```

### useChartData

**Location**: `src/hooks/use-chart-data.ts`

**Purpose**: Filters and processes chart data based on various criteria.

**Parameters**:
- `data: ChartDataPoint[]` - Array of chart data points
- `filters?: ChartDataFilters` - Optional filters object

**Filters Object**:
- `limit?: number` - Maximum number of items to return
- `labelFilter?: string` - Filter by specific label
- `dateRange?: { start: Date; end: Date }` - Filter by date range

**Returns**: Filtered `ChartDataPoint[]` array

**Example Usage**:
```typescript
const incomeData = useChartData(mockIncomeExpenseHistory, {
  labelFilter: "Income",
  limit: 6,
});
```

### useChartStats

**Location**: `src/hooks/use-chart-data.ts`

**Purpose**: Calculates statistics from chart data.

**Parameters**:
- `data: ChartDataPoint[]` - Array of chart data points

**Returns**: Statistics object containing:
- `total: number` - Sum of all values
- `average: number` - Average value
- `min: number` - Minimum value
- `max: number` - Maximum value
- `trend: 'up' | 'down' | 'neutral'` - Trend direction based on first/last values

## Benefits

### Improved Testability
- Each calculation is isolated and can be tested independently
- Comprehensive test coverage for all edge cases
- Easier to mock and test different scenarios

### Better Maintainability
- Calculations are centralized and reusable
- Clear separation of concerns
- Easier to modify calculation logic without touching UI components

### Enhanced Reusability
- Hooks can be used in multiple components
- Consistent calculation logic across the application
- Easier to extend with additional metrics

## Migration

The `DashboardPage` component has been updated to use these hooks:

**Before**:
```typescript
// Inline calculations
const netWorth = stats.totalBalance;
const monthlyNet = stats.monthlyIncome - stats.monthlyExpenses;
const expenseRatio = (stats.monthlyExpenses / stats.monthlyIncome) * 100;

const budgetSummary = mockBudgets.reduce((acc, budget) => {
  // Complex calculation logic...
}, { onTrack: 0, warning: 0, overLimit: 0 });
```

**After**:
```typescript
// Using custom hooks
const dashboardMetrics = useDashboardMetrics(stats);
const budgetSummary = useBudgetSummary(mockBudgets);
const incomeChartData = useChartData(mockIncomeExpenseHistory, {
  labelFilter: "Income",
  limit: 6,
});
```

## Testing

All hooks include comprehensive test suites covering:
- Normal operation scenarios
- Edge cases (empty data, null values, zero values)
- Memoization behavior
- Recalculation when dependencies change

Test files are located in `src/hooks/__tests__/` directory.
