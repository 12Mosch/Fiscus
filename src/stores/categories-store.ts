/**
 * Categories store using Zustand
 * Manages category data and provides category-related actions
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	Category,
	CategoryFilters,
	CreateCategoryRequest,
	UpdateCategoryRequest,
} from "../types/api";

interface CategoriesState {
	/** List of categories */
	categories: Category[];
	/** Currently selected category */
	selectedCategory: Category | null;
	/** Loading state for category operations */
	loading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether categories have been loaded */
	initialized: boolean;
}

interface CategoriesActions {
	/** Load categories with filters */
	loadCategories: (filters: CategoryFilters) => Promise<void>;
	/** Load category hierarchy */
	loadCategoryHierarchy: (userId: string, isIncome?: boolean) => Promise<void>;
	/** Create a new category */
	createCategory: (request: CreateCategoryRequest) => Promise<Category | null>;
	/** Update a category */
	updateCategory: (
		categoryId: string,
		userId: string,
		request: UpdateCategoryRequest,
	) => Promise<Category | null>;
	/** Delete a category */
	deleteCategory: (categoryId: string, userId: string) => Promise<boolean>;
	/** Select a category */
	selectCategory: (category: Category | null) => void;
	/** Get category by ID */
	getCategoryById: (categoryId: string) => Category | null;
	/** Refresh categories data */
	refreshCategories: (filters: CategoryFilters) => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Reset store state */
	reset: () => void;
}

export type CategoriesStore = CategoriesState & CategoriesActions;

export const useCategoriesStore = create<CategoriesStore>()((set, get) => ({
	// Initial state
	categories: [],
	selectedCategory: null,
	loading: false,
	error: null,
	initialized: false,

	// Actions
	loadCategories: async (filters: CategoryFilters): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const categories = await apiClient.getCategories(filters);

			set({
				categories,
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
						: new FiscusApiError("Failed to load categories", "INTERNAL_ERROR"),
			});
		}
	},

	loadCategoryHierarchy: async (
		userId: string,
		isIncome?: boolean,
	): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const categories = await apiClient.getCategoryHierarchy(userId, isIncome);

			set({
				categories,
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
						: new FiscusApiError(
								"Failed to load category hierarchy",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	createCategory: async (
		request: CreateCategoryRequest,
	): Promise<Category | null> => {
		set({ loading: true, error: null });

		try {
			const newCategory = await apiClient.createCategory(request);

			// Add the new category to the list
			set((state) => ({
				categories: [...state.categories, newCategory],
				loading: false,
				error: null,
			}));

			return newCategory;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to create category", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	updateCategory: async (
		categoryId: string,
		userId: string,
		request: UpdateCategoryRequest,
	): Promise<Category | null> => {
		set({ loading: true, error: null });

		try {
			const updatedCategory = await apiClient.updateCategory(
				categoryId,
				userId,
				request,
			);

			// Update the category in the list
			set((state) => ({
				categories: state.categories.map((category) =>
					category.id === categoryId ? updatedCategory : category,
				),
				selectedCategory:
					state.selectedCategory?.id === categoryId
						? updatedCategory
						: state.selectedCategory,
				loading: false,
				error: null,
			}));

			return updatedCategory;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to update category", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	deleteCategory: async (
		categoryId: string,
		userId: string,
	): Promise<boolean> => {
		set({ loading: true, error: null });

		try {
			const success = await apiClient.deleteCategory(categoryId, userId);

			if (success) {
				// Remove the category from the list
				set((state) => ({
					categories: state.categories.filter(
						(category) => category.id !== categoryId,
					),
					selectedCategory:
						state.selectedCategory?.id === categoryId
							? null
							: state.selectedCategory,
					loading: false,
					error: null,
				}));
			} else {
				set({
					loading: false,
					error: new FiscusApiError(
						"Category deletion failed - it may have associated transactions",
						"VALIDATION_ERROR",
					),
				});
			}

			return success;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to delete category", "INTERNAL_ERROR"),
			});

			return false;
		}
	},

	selectCategory: (category: Category | null) => {
		set({ selectedCategory: category });
	},

	getCategoryById: (categoryId: string): Category | null => {
		const { categories } = get();
		return categories.find((category) => category.id === categoryId) || null;
	},

	refreshCategories: async (filters: CategoryFilters): Promise<void> => {
		await get().loadCategories(filters);
	},

	clearError: () => {
		set({ error: null });
	},

	setLoading: (loading: boolean) => {
		set({ loading });
	},

	reset: () => {
		set({
			categories: [],
			selectedCategory: null,
			loading: false,
			error: null,
			initialized: false,
		});
	},
}));

/**
 * Selector hooks for common category state
 */
export const useCategories = () => {
	const { categories, loading, error } = useCategoriesStore();
	return { categories, loading, error };
};

export const useCategoriesActions = () => {
	const {
		loadCategories,
		loadCategoryHierarchy,
		createCategory,
		updateCategory,
		deleteCategory,
		refreshCategories,
		clearError,
	} = useCategoriesStore();
	return {
		loadCategories,
		loadCategoryHierarchy,
		createCategory,
		updateCategory,
		deleteCategory,
		refreshCategories,
		clearError,
	};
};

export const useSelectedCategory = () => {
	const { selectedCategory, selectCategory } = useCategoriesStore();
	return { selectedCategory, selectCategory };
};

/**
 * Hook to get categories by type (income/expense)
 */
export const useCategoriesByType = (isIncome: boolean) => {
	const categories = useCategoriesStore((state) => state.categories);
	return categories.filter((category) => category.is_income === isIncome);
};

/**
 * Hook to get active categories only
 */
export const useActiveCategories = () => {
	const categories = useCategoriesStore((state) => state.categories);
	return categories.filter((category) => category.is_active);
};

/**
 * Hook to get root categories (no parent)
 */
export const useRootCategories = () => {
	const categories = useCategoriesStore((state) => state.categories);
	return categories.filter((category) => !category.parent_category_id);
};

/**
 * Hook to get subcategories of a parent category
 */
export const useSubcategories = (parentCategoryId: string) => {
	const categories = useCategoriesStore((state) => state.categories);
	return categories.filter(
		(category) => category.parent_category_id === parentCategoryId,
	);
};

/**
 * Hook to get category by ID with reactive updates
 */
export const useCategoryById = (categoryId: string) => {
	return useCategoriesStore(
		(state) =>
			state.categories.find((category) => category.id === categoryId) || null,
	);
};
