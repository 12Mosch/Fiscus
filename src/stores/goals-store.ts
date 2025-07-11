/**
 * Goals store using Zustand
 * Manages financial goals data and provides goal-related actions
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	CreateGoalRequest,
	Goal,
	GoalFilters,
	GoalStatus,
	ReportData,
	UpdateGoalRequest,
} from "../types/api";

interface GoalsState {
	/** List of goals */
	goals: Goal[];
	/** Currently selected goal */
	selectedGoal: Goal | null;
	/** Goal progress summary */
	progressSummary: ReportData | null;
	/** Loading state for goal operations */
	loading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether goals have been loaded */
	initialized: boolean;
}

interface GoalsActions {
	/** Load goals with filters */
	loadGoals: (filters: GoalFilters) => Promise<void>;
	/** Create a new goal */
	createGoal: (request: CreateGoalRequest) => Promise<Goal | null>;
	/** Update a goal */
	updateGoal: (
		goalId: string,
		userId: string,
		request: UpdateGoalRequest,
	) => Promise<Goal | null>;
	/** Delete a goal */
	deleteGoal: (goalId: string, userId: string) => Promise<boolean>;
	/** Update goal progress */
	updateGoalProgress: (
		goalId: string,
		userId: string,
		amount: number,
	) => Promise<Goal | null>;
	/** Load goal progress summary */
	loadGoalProgressSummary: (userId: string) => Promise<void>;
	/** Select a goal */
	selectGoal: (goal: Goal | null) => void;
	/** Get goal by ID */
	getGoalById: (goalId: string) => Goal | null;
	/** Refresh goals data */
	refreshGoals: (filters: GoalFilters) => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Reset store state */
	reset: () => void;
}

export type GoalsStore = GoalsState & GoalsActions;

export const useGoalsStore = create<GoalsStore>()((set, get) => ({
	// Initial state
	goals: [],
	selectedGoal: null,
	progressSummary: null,
	loading: false,
	error: null,
	initialized: false,

	// Actions
	loadGoals: async (filters: GoalFilters): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const goals = await apiClient.getGoals(filters);

			set({
				goals,
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
						: new FiscusApiError("Failed to load goals", "INTERNAL_ERROR"),
			});
		}
	},

	createGoal: async (request: CreateGoalRequest): Promise<Goal | null> => {
		set({ loading: true, error: null });

		try {
			const newGoal = await apiClient.createGoal(request);

			// Add the new goal to the list
			set((state) => ({
				goals: [...state.goals, newGoal],
				loading: false,
				error: null,
			}));

			// Refresh progress summary if it exists
			if (get().progressSummary) {
				get().loadGoalProgressSummary(request.user_id);
			}

			return newGoal;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to create goal", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	updateGoal: async (
		goalId: string,
		userId: string,
		request: UpdateGoalRequest,
	): Promise<Goal | null> => {
		set({ loading: true, error: null });

		try {
			const updatedGoal = await apiClient.updateGoal(goalId, userId, request);

			// Update the goal in the list
			set((state) => ({
				goals: state.goals.map((goal) =>
					goal.id === goalId ? updatedGoal : goal,
				),
				selectedGoal:
					state.selectedGoal?.id === goalId ? updatedGoal : state.selectedGoal,
				loading: false,
				error: null,
			}));

			// Refresh progress summary if it exists
			if (get().progressSummary) {
				get().loadGoalProgressSummary(userId);
			}

			return updatedGoal;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to update goal", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	deleteGoal: async (goalId: string, userId: string): Promise<boolean> => {
		set({ loading: true, error: null });

		try {
			const success = await apiClient.deleteGoal(goalId, userId);

			if (success) {
				// Remove the goal from the list
				set((state) => ({
					goals: state.goals.filter((goal) => goal.id !== goalId),
					selectedGoal:
						state.selectedGoal?.id === goalId ? null : state.selectedGoal,
					loading: false,
					error: null,
				}));

				// Refresh progress summary if it exists
				if (get().progressSummary) {
					get().loadGoalProgressSummary(userId);
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
						: new FiscusApiError("Failed to delete goal", "INTERNAL_ERROR"),
			});

			return false;
		}
	},

	updateGoalProgress: async (
		goalId: string,
		userId: string,
		amount: number,
	): Promise<Goal | null> => {
		set({ loading: true, error: null });

		try {
			const updatedGoal = await apiClient.updateGoalProgress(
				goalId,
				userId,
				amount,
			);

			// Update the goal in the list
			set((state) => ({
				goals: state.goals.map((goal) =>
					goal.id === goalId ? updatedGoal : goal,
				),
				selectedGoal:
					state.selectedGoal?.id === goalId ? updatedGoal : state.selectedGoal,
				loading: false,
				error: null,
			}));

			// Refresh progress summary if it exists
			if (get().progressSummary) {
				get().loadGoalProgressSummary(userId);
			}

			return updatedGoal;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to update goal progress",
								"INTERNAL_ERROR",
							),
			});

			return null;
		}
	},

	loadGoalProgressSummary: async (userId: string): Promise<void> => {
		try {
			const progressSummary = await apiClient.getGoalProgressSummary(userId);

			set({
				progressSummary,
				error: null,
			});
		} catch (error) {
			set({
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load goal progress summary",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	selectGoal: (goal: Goal | null) => {
		set({ selectedGoal: goal });
	},

	getGoalById: (goalId: string): Goal | null => {
		const { goals } = get();
		return goals.find((goal) => goal.id === goalId) || null;
	},

	refreshGoals: async (filters: GoalFilters): Promise<void> => {
		await get().loadGoals(filters);
	},

	clearError: () => {
		set({ error: null });
	},

	setLoading: (loading: boolean) => {
		set({ loading });
	},

	reset: () => {
		set({
			goals: [],
			selectedGoal: null,
			progressSummary: null,
			loading: false,
			error: null,
			initialized: false,
		});
	},
}));

/**
 * Selector hooks for common goal state
 */
export const useGoals = () => {
	const { goals, loading, error } = useGoalsStore();
	return { goals, loading, error };
};

export const useGoalsActions = () => {
	const {
		loadGoals,
		createGoal,
		updateGoal,
		deleteGoal,
		updateGoalProgress,
		refreshGoals,
		clearError,
	} = useGoalsStore();
	return {
		loadGoals,
		createGoal,
		updateGoal,
		deleteGoal,
		updateGoalProgress,
		refreshGoals,
		clearError,
	};
};

export const useSelectedGoal = () => {
	const { selectedGoal, selectGoal } = useGoalsStore();
	return { selectedGoal, selectGoal };
};

export const useGoalProgressSummary = () => {
	const { progressSummary, loadGoalProgressSummary } = useGoalsStore();
	return { progressSummary, loadGoalProgressSummary };
};

/**
 * Hook to get goals by status
 */
export const useGoalsByStatus = (status: GoalStatus) => {
	const goals = useGoalsStore((state) => state.goals);
	return goals.filter((goal) => goal.status === status);
};

/**
 * Hook to get active goals only
 */
export const useActiveGoals = () => {
	const goals = useGoalsStore((state) => state.goals);
	return goals.filter((goal) => goal.status === "active");
};

/**
 * Hook to get completed goals
 */
export const useCompletedGoals = () => {
	const goals = useGoalsStore((state) => state.goals);
	return goals.filter((goal) => goal.status === "completed");
};

/**
 * Hook to get goals by category
 */
export const useGoalsByCategory = (category: string) => {
	const goals = useGoalsStore((state) => state.goals);
	return goals.filter((goal) => goal.category === category);
};

/**
 * Hook to get goal progress percentage
 */
export const useGoalProgress = (goalId: string): number => {
	const goal = useGoalsStore((state) =>
		state.goals.find((g) => g.id === goalId),
	);

	if (!goal || goal.target_amount === 0) return 0;

	return Math.min((goal.current_amount / goal.target_amount) * 100, 100);
};

/**
 * Hook to get goal by ID with reactive updates
 */
export const useGoalById = (goalId: string) => {
	return useGoalsStore(
		(state) => state.goals.find((goal) => goal.id === goalId) || null,
	);
};
