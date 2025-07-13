/**
 * Tests for authentication store
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { apiClient, FiscusApiError } from "../../api/client";
import type { CreateUserRequest, LoginRequest, User } from "../../types/api";
import { useAuthStore } from "../auth-store";

// Mock the API client
vi.mock("../../api/client", () => ({
	apiClient: {
		loginUser: vi.fn(),
		createUser: vi.fn(),
		changePassword: vi.fn(),
		getCurrentUser: vi.fn(),
	},
	FiscusApiError: class MockFiscusApiError extends Error {
		constructor(
			message: string,
			public code: string,
			public statusCode?: number,
		) {
			super(message);
			this.name = "FiscusApiError";
		}
	},
}));

const mockApiClient = apiClient as unknown as {
	loginUser: ReturnType<typeof vi.fn>;
	createUser: ReturnType<typeof vi.fn>;
	changePassword: ReturnType<typeof vi.fn>;
	getCurrentUser: ReturnType<typeof vi.fn>;
};

describe("AuthStore", () => {
	beforeEach(() => {
		// Reset store state before each test
		useAuthStore.getState().logout();
		// Reset initialized state for testing
		useAuthStore.setState({ initialized: false });
		vi.clearAllMocks();
	});

	afterEach(() => {
		// Clean up after each test
		useAuthStore.getState().logout();
	});

	describe("Initial State", () => {
		it("should have correct initial state", () => {
			// Reset to truly initial state for this test
			useAuthStore.setState({
				user: null,
				isAuthenticated: false,
				loading: false,
				error: null,
				initialized: false,
			});

			const state = useAuthStore.getState();

			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.loading).toBe(false);
			expect(state.error).toBeNull();
			expect(state.initialized).toBe(false);
		});
	});

	describe("Login", () => {
		it("should login successfully", async () => {
			const mockUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "testuser",
				email: "test@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			const loginRequest: LoginRequest = {
				// file deepcode ignore NoHardcodedCredentials/test: <test>
				username: "testuser",
				// file deepcode ignore NoHardcodedPasswords/test: <test>
				password: "password123",
			};

			mockApiClient.loginUser.mockResolvedValue({
				user: mockUser,
				session_token: "mock-token",
			});

			const result = await useAuthStore.getState().login(loginRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(true);
			expect(state.user).toEqual(mockUser);
			expect(state.isAuthenticated).toBe(true);
			expect(state.loading).toBe(false);
			expect(state.error).toBeNull();
		});

		it("should handle login failure", async () => {
			const loginRequest: LoginRequest = {
				username: "testuser",
				password: "wrongpassword",
			};

			const mockError = new Error("Invalid credentials");
			mockApiClient.loginUser.mockRejectedValue(mockError);

			const result = await useAuthStore.getState().login(loginRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(false);
			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.loading).toBe(false);
			expect(state.error).toBeTruthy();
		});

		it("should set loading state during login", async () => {
			const loginRequest: LoginRequest = {
				username: "testuser",
				password: "password123",
			};

			// Create a promise that we can control
			let resolveLogin!: (value: { user: User }) => void;
			const loginPromise = new Promise<{ user: User }>((resolve) => {
				resolveLogin = resolve;
			});

			mockApiClient.loginUser.mockReturnValue(loginPromise);

			// Start login
			const loginCall = useAuthStore.getState().login(loginRequest);

			// Check loading state
			expect(useAuthStore.getState().loading).toBe(true);

			// Resolve the login
			resolveLogin({
				user: {
					id: "123e4567-e89b-12d3-a456-426614174000",
					username: "testuser",
					email: "test@example.com",
					created_at: "2024-01-01T00:00:00Z",
					updated_at: "2024-01-01T00:00:00Z",
				},
			});

			await loginCall;

			// Check loading state is cleared
			expect(useAuthStore.getState().loading).toBe(false);
		});
	});

	describe("Register", () => {
		it("should register and auto-login successfully", async () => {
			const mockUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "newuser",
				email: "new@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			const registerRequest: CreateUserRequest = {
				username: "newuser",
				email: "new@example.com",
				password: "password123",
			};

			mockApiClient.createUser.mockResolvedValue(mockUser);
			mockApiClient.loginUser.mockResolvedValue({
				user: mockUser,
				session_token: "mock-token",
			});

			const result = await useAuthStore.getState().register(registerRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(true);
			expect(state.user).toEqual(mockUser);
			expect(state.isAuthenticated).toBe(true);
			expect(mockApiClient.createUser).toHaveBeenCalledWith(registerRequest);
			expect(mockApiClient.loginUser).toHaveBeenCalledWith({
				username: registerRequest.username,
				password: registerRequest.password,
			});
		});

		it("should handle registration failure", async () => {
			const registerRequest: CreateUserRequest = {
				username: "newuser",
				email: "new@example.com",
				password: "password123",
			};

			const mockError = new Error("Username already exists");
			mockApiClient.createUser.mockRejectedValue(mockError);

			const result = await useAuthStore.getState().register(registerRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(false);
			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.error).toBeTruthy();
		});
	});

	describe("Logout", () => {
		it("should logout and clear state", async () => {
			// First login
			const mockUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "testuser",
				email: "test@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			mockApiClient.loginUser.mockResolvedValue({
				user: mockUser,
				session_token: "mock-token",
			});

			await useAuthStore.getState().login({
				username: "testuser",
				password: "password123",
			});

			// Verify logged in
			expect(useAuthStore.getState().isAuthenticated).toBe(true);

			// Logout
			useAuthStore.getState().logout();

			const state = useAuthStore.getState();
			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.error).toBeNull();
			expect(state.loading).toBe(false);
		});
	});

	describe("Change Password", () => {
		it("should change password successfully", async () => {
			const changePasswordRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				current_password: "oldpassword",
				new_password: "newpassword",
			};

			mockApiClient.changePassword.mockResolvedValue(true);

			const result = await useAuthStore
				.getState()
				.changePassword(changePasswordRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(true);
			expect(state.loading).toBe(false);
			expect(state.error).toBeNull();
		});

		it("should handle password change failure", async () => {
			const changePasswordRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				current_password: "wrongpassword",
				new_password: "newpassword",
			};

			const mockError = new Error("Current password is incorrect");
			mockApiClient.changePassword.mockRejectedValue(mockError);

			const result = await useAuthStore
				.getState()
				.changePassword(changePasswordRequest);
			const state = useAuthStore.getState();

			expect(result).toBe(false);
			expect(state.loading).toBe(false);
			expect(state.error).toBeTruthy();
		});
	});

	describe("Refresh User", () => {
		it("should refresh user data successfully", async () => {
			// Setup initial user
			const initialUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "testuser",
				email: "test@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			useAuthStore.setState({
				user: initialUser,
				isAuthenticated: true,
			});

			// Setup updated user data
			const updatedUser: User = {
				...initialUser,
				email: "updated@example.com",
				updated_at: "2024-01-01T01:00:00Z",
			};

			mockApiClient.getCurrentUser.mockResolvedValue(updatedUser);

			await useAuthStore.getState().refreshUser();
			const state = useAuthStore.getState();

			expect(state.user).toEqual(updatedUser);
			expect(state.loading).toBe(false);
			expect(state.error).toBeNull();
		});

		it("should not refresh when no user is logged in", async () => {
			await useAuthStore.getState().refreshUser();

			expect(mockApiClient.getCurrentUser).not.toHaveBeenCalled();
		});
	});

	describe("Error Handling", () => {
		it("should clear error state", () => {
			// Set an error
			useAuthStore.setState({
				error: new FiscusApiError("Test error", "TEST_ERROR"),
			});

			expect(useAuthStore.getState().error).toBeTruthy();

			// Clear error
			useAuthStore.getState().clearError();

			expect(useAuthStore.getState().error).toBeNull();
		});

		it("should set loading state", () => {
			useAuthStore.getState().setLoading(true);
			expect(useAuthStore.getState().loading).toBe(true);

			useAuthStore.getState().setLoading(false);
			expect(useAuthStore.getState().loading).toBe(false);
		});
	});

	describe("Initialize", () => {
		it("should initialize with valid stored user", async () => {
			const mockUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "testuser",
				email: "test@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			// Setup stored user
			useAuthStore.setState({
				user: mockUser,
			});

			mockApiClient.getCurrentUser.mockResolvedValue(mockUser);

			await useAuthStore.getState().initialize();
			const state = useAuthStore.getState();

			expect(state.isAuthenticated).toBe(true);
			expect(state.initialized).toBe(true);
		});

		it("should clear invalid stored user", async () => {
			const mockUser: User = {
				id: "123e4567-e89b-12d3-a456-426614174000",
				username: "testuser",
				email: "test@example.com",
				created_at: "2024-01-01T00:00:00Z",
				updated_at: "2024-01-01T00:00:00Z",
			};

			// Setup stored user
			useAuthStore.setState({
				user: mockUser,
			});

			mockApiClient.getCurrentUser.mockRejectedValue(
				new Error("User not found"),
			);

			await useAuthStore.getState().initialize();
			const state = useAuthStore.getState();

			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.initialized).toBe(true);
		});
	});
});
