/**
 * Budgets store using Zustand
 * Manages budget and budget period data with actions
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	Budget,
	BudgetFilters,
	BudgetPeriod,
	BudgetSummaryResponse,
	CreateBudgetPeriodRequest,
	CreateBudgetRequest,
	UpdateBudgetRequest,
} from "../types/api";

interface BudgetsState {
	/** List of budgets */
	budgets: Budget[];
	/** List of budget periods */
	budgetPeriods: BudgetPeriod[];
	/** Currently selected budget */
	selectedBudget: Budget | null;
	/** Currently selected budget period */
	selectedBudgetPeriod: BudgetPeriod | null;
	/** Budget summary data */
	summary: BudgetSummaryResponse | null;
	/** Loading state for budget operations */
	loading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether budgets have been loaded */
	initialized: boolean;
}

interface BudgetsActions {
	/** Load budgets with filters */
	loadBudgets: (filters: BudgetFilters) => Promise<void>;
	/** Load budget periods */
	loadBudgetPeriods: (userId: string, isActive?: boolean) => Promise<void>;
	/** Create a new budget period */
	createBudgetPeriod: (
		request: CreateBudgetPeriodRequest,
	) => Promise<BudgetPeriod | null>;
	/** Create a new budget */
	createBudget: (request: CreateBudgetRequest) => Promise<Budget | null>;
	/** Update a budget */
	updateBudget: (
		budgetId: string,
		userId: string,
		request: UpdateBudgetRequest,
	) => Promise<Budget | null>;
	/** Delete a budget */
	deleteBudget: (budgetId: string, userId: string) => Promise<boolean>;
	/** Load budget summary */
	loadBudgetSummary: (userId: string, budgetPeriodId?: string) => Promise<void>;
	/** Select a budget */
	selectBudget: (budget: Budget | null) => void;
	/** Select a budget period */
	selectBudgetPeriod: (budgetPeriod: BudgetPeriod | null) => void;
	/** Get budget by ID */
	getBudgetById: (budgetId: string) => Budget | null;
	/** Refresh budgets data */
	refreshBudgets: (filters: BudgetFilters) => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Reset store state */
	reset: () => void;
}

export type BudgetsStore = BudgetsState & BudgetsActions;

export const useBudgetsStore = create<BudgetsStore>()((set, get) => ({
	// Initial state
	budgets: [],
	budgetPeriods: [],
	selectedBudget: null,
	selectedBudgetPeriod: null,
	summary: null,
	loading: false,
	error: null,
	initialized: false,

	// Actions
	loadBudgets: async (filters: BudgetFilters): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const budgets = await apiClient.getBudgets(filters);

			set({
				budgets,
				loading: false,
				error: null,
				initialized: true,
			});
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to load budgets", "INTERNAL_ERROR"),
			});
		}
	},

	loadBudgetPeriods: async (
		userId: string,
		isActive?: boolean,
	): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const budgetPeriods = await apiClient.getBudgetPeriods(userId, isActive);

			set({
				budgetPeriods,
				loading: false,
				error: null,
			});
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load budget periods",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	createBudgetPeriod: async (
		request: CreateBudgetPeriodRequest,
	): Promise<BudgetPeriod | null> => {
		set({ loading: true, error: null });

		try {
			const newBudgetPeriod = await apiClient.createBudgetPeriod(request);

			// Add the new budget period to the list
			set((state) => ({
				budgetPeriods: [...state.budgetPeriods, newBudgetPeriod],
				loading: false,
				error: null,
			}));

			return newBudgetPeriod;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to create budget period",
								"INTERNAL_ERROR",
							),
			});

			return null;
		}
	},

	createBudget: async (
		request: CreateBudgetRequest,
	): Promise<Budget | null> => {
		set({ loading: true, error: null });

		try {
			const newBudget = await apiClient.createBudget(request);

			// Add the new budget to the list
			set((state) => ({
				budgets: [...state.budgets, newBudget],
				loading: false,
				error: null,
			}));

			// Refresh summary if it exists
			if (get().summary) {
				get()
					.loadBudgetSummary(request.user_id, request.budget_period_id)
					.catch(() => {
						// Errors are already handled in loadBudgetSummary, but this prevents unhandled rejection warnings
					});
			}

			return newBudget;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to create budget", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	updateBudget: async (
		budgetId: string,
		userId: string,
		request: UpdateBudgetRequest,
	): Promise<Budget | null> => {
		set({ loading: true, error: null });

		try {
			const updatedBudget = await apiClient.updateBudget(
				budgetId,
				userId,
				request,
			);

			// Update the budget in the list
			set((state) => ({
				budgets: state.budgets.map((budget) =>
					budget.id === budgetId ? updatedBudget : budget,
				),
				selectedBudget:
					state.selectedBudget?.id === budgetId
						? updatedBudget
						: state.selectedBudget,
				loading: false,
				error: null,
			}));

			// Refresh summary if it exists
			if (get().summary) {
				get()
					.loadBudgetSummary(userId, updatedBudget.budget_period_id)
					.catch(() => {
						// Errors are already handled in loadBudgetSummary
					});
			}

			return updatedBudget;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to update budget", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	deleteBudget: async (budgetId: string, userId: string): Promise<boolean> => {
		set({ loading: true, error: null });

		try {
			const success = await apiClient.deleteBudget(budgetId, userId);

			if (success) {
				// Remove the budget from the list
				set((state) => ({
					budgets: state.budgets.filter((budget) => budget.id !== budgetId),
					selectedBudget:
						state.selectedBudget?.id === budgetId ? null : state.selectedBudget,
					loading: false,
					error: null,
				}));

				// Refresh summary if it exists
				if (get().summary) {
					get()
						.loadBudgetSummary(userId)
						.catch(() => {
							// Errors are already handled in loadBudgetSummary
						});
				}
			} else {
				set({ loading: false });
			}

			return success;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to delete budget", "INTERNAL_ERROR"),
			});

			return false;
		}
	},

	loadBudgetSummary: async (
		userId: string,
		budgetPeriodId?: string,
	): Promise<void> => {
		try {
			const summary = await apiClient.getBudgetSummary(userId, budgetPeriodId);

			set({
				summary,
				error: null,
			});
		} catch (error) {
			set({
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load budget summary",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	selectBudget: (budget: Budget | null) => {
		set({ selectedBudget: budget });
	},

	selectBudgetPeriod: (budgetPeriod: BudgetPeriod | null) => {
		set({ selectedBudgetPeriod: budgetPeriod });
	},

	getBudgetById: (budgetId: string): Budget | null => {
		const { budgets } = get();
		return budgets.find((budget) => budget.id === budgetId) || null;
	},

	refreshBudgets: async (filters: BudgetFilters): Promise<void> => {
		await get().loadBudgets(filters);
	},

	clearError: () => {
		set({ error: null });
	},

	setLoading: (loading: boolean) => {
		set({ loading });
	},

	reset: () => {
		set({
			budgets: [],
			budgetPeriods: [],
			selectedBudget: null,
			selectedBudgetPeriod: null,
			summary: null,
			loading: false,
			error: null,
			initialized: false,
		});
	},
}));

/**
 * Selector hooks for common budget state
 */
export const useBudgets = () => {
	const { budgets, loading, error } = useBudgetsStore();
	return { budgets, loading, error };
};

export const useBudgetPeriods = () => {
	const { budgetPeriods, loading, error } = useBudgetsStore();
	return { budgetPeriods, loading, error };
};

export const useBudgetsActions = () => {
	const {
		loadBudgets,
		loadBudgetPeriods,
		createBudgetPeriod,
		createBudget,
		updateBudget,
		deleteBudget,
		refreshBudgets,
		clearError,
	} = useBudgetsStore();
	return {
		loadBudgets,
		loadBudgetPeriods,
		createBudgetPeriod,
		createBudget,
		updateBudget,
		deleteBudget,
		refreshBudgets,
		clearError,
	};
};

export const useSelectedBudget = () => {
	const { selectedBudget, selectBudget } = useBudgetsStore();
	return { selectedBudget, selectBudget };
};

export const useSelectedBudgetPeriod = () => {
	const { selectedBudgetPeriod, selectBudgetPeriod } = useBudgetsStore();
	return { selectedBudgetPeriod, selectBudgetPeriod };
};

export const useBudgetSummary = () => {
	const { summary, loadBudgetSummary } = useBudgetsStore();
	return { summary, loadBudgetSummary };
};

/**
 * Hook to get budgets by period
 */
export const useBudgetsByPeriod = (budgetPeriodId: string) => {
	const budgets = useBudgetsStore((state) => state.budgets);
	return budgets.filter((budget) => budget.budget_period_id === budgetPeriodId);
};

/**
 * Hook to get active budget periods
 */
export const useActiveBudgetPeriods = () => {
	const budgetPeriods = useBudgetsStore((state) => state.budgetPeriods);
	return budgetPeriods.filter((period) => period.is_active);
};

/**
 * Hook to get budget by ID with reactive updates
 */
export const useBudgetById = (budgetId: string) => {
	return useBudgetsStore(
		(state) => state.budgets.find((budget) => budget.id === budgetId) || null,
	);
};
