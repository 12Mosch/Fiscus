/**
 * Authentication store using Zustand
 * Manages user authentication state and provides auth-related actions
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	ChangePasswordRequest,
	CreateUserRequest,
	LoginRequest,
	User,
} from "../types/api";

interface AuthState {
	/** Current authenticated user */
	user: User | null;
	/** Whether user is authenticated */
	isAuthenticated: boolean;
	/** Loading state for auth operations */
	loading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether the auth state has been initialized */
	initialized: boolean;
}

interface AuthActions {
	/** Login user with credentials */
	login: (credentials: LoginRequest) => Promise<boolean>;
	/** Register new user */
	register: (userData: CreateUserRequest) => Promise<boolean>;
	/** Logout current user */
	logout: () => void;
	/** Change user password */
	changePassword: (request: ChangePasswordRequest) => Promise<boolean>;
	/** Refresh user data */
	refreshUser: () => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Initialize auth state */
	initialize: () => Promise<void>;
}

export type AuthStore = AuthState & AuthActions;

export const useAuthStore = create<AuthStore>()(
	persist(
		(set, get) => ({
			// Initial state
			user: null,
			isAuthenticated: false,
			loading: false,
			error: null,
			initialized: false,

			// Actions
			login: async (credentials: LoginRequest): Promise<boolean> => {
				set({ loading: true, error: null });

				try {
					const response = await apiClient.loginUser(credentials);

					set({
						user: response.user,
						isAuthenticated: true,
						loading: false,
						error: null,
					});

					return true;
				} catch (error) {
					set({
						loading: false,
						error:
							error instanceof FiscusApiError
								? error
								: new FiscusApiError("Login failed", "AUTHENTICATION_ERROR"),
						user: null,
						isAuthenticated: false,
					});

					return false;
				}
			},

			register: async (userData: CreateUserRequest): Promise<boolean> => {
				set({ loading: true, error: null });

				try {
					await apiClient.createUser(userData);

					// Auto-login after successful registration
					const loginSuccess = await get().login({
						username: userData.username,
						password: userData.password,
					});

					if (!loginSuccess) {
						// If auto-login fails, still consider registration successful
						set({
							loading: false,
							error: null,
						});
					}

					return true;
				} catch (error) {
					set({
						loading: false,
						error:
							error instanceof FiscusApiError
								? error
								: new FiscusApiError("Registration failed", "VALIDATION_ERROR"),
					});

					return false;
				}
			},

			logout: () => {
				set({
					user: null,
					isAuthenticated: false,
					error: null,
					loading: false,
				});
			},

			changePassword: async (
				request: ChangePasswordRequest,
			): Promise<boolean> => {
				set({ loading: true, error: null });

				try {
					const success = await apiClient.changePassword(request);

					set({
						loading: false,
						error: null,
					});

					return success;
				} catch (error) {
					set({
						loading: false,
						error:
							error instanceof FiscusApiError
								? error
								: new FiscusApiError(
										"Password change failed",
										"AUTHENTICATION_ERROR",
									),
					});

					return false;
				}
			},

			refreshUser: async (): Promise<void> => {
				const { user } = get();
				if (!user) return;

				set({ loading: true, error: null });

				try {
					const updatedUser = await apiClient.getCurrentUser(user.id);

					set({
						user: updatedUser,
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
										"Failed to refresh user data",
										"INTERNAL_ERROR",
									),
					});
				}
			},

			clearError: () => {
				set({ error: null });
			},

			setLoading: (loading: boolean) => {
				set({ loading });
			},

			initialize: async (): Promise<void> => {
				const { user } = get();

				if (user) {
					// Verify the stored user is still valid
					try {
						await apiClient.getCurrentUser(user.id);
						set({
							isAuthenticated: true,
							initialized: true,
						});
					} catch {
						// User is no longer valid, clear auth state
						set({
							user: null,
							isAuthenticated: false,
							initialized: true,
							error: null,
						});
					}
				} else {
					set({ initialized: true });
				}
			},
		}),
		{
			name: "fiscus-auth-storage",
			partialize: (state) => ({
				user: state.user,
				isAuthenticated: state.isAuthenticated,
			}),
			onRehydrateStorage: () => (state) => {
				if (state) {
					// Initialize auth state after rehydration
					state.initialize?.();
				}
			},
		},
	),
);

/**
 * Selector hooks for common auth state
 */
export const useAuth = () => {
	const { user, isAuthenticated, loading, error } = useAuthStore();
	return { user, isAuthenticated, loading, error };
};

export const useAuthActions = () => {
	const { login, register, logout, changePassword, refreshUser, clearError } =
		useAuthStore();
	return { login, register, logout, changePassword, refreshUser, clearError };
};

/**
 * Hook to get current user ID safely
 */
export const useUserId = (): string | null => {
	const user = useAuthStore((state) => state.user);
	return user?.id || null;
};

/**
 * Hook to check if user is authenticated
 */
export const useIsAuthenticated = (): boolean => {
	return useAuthStore((state) => state.isAuthenticated);
};

/**
 * Hook to get auth loading state
 */
export const useAuthLoading = (): boolean => {
	return useAuthStore((state) => state.loading);
};

/**
 * Hook to get auth error
 */
export const useAuthError = (): FiscusApiError | null => {
	return useAuthStore((state) => state.error);
};
