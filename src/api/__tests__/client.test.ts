/**
 * Tests for API client
 */

import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CreateUserRequest, LoginRequest, User } from "../../types/api";
import { FiscusApiClient, FiscusApiError } from "../client";

// Mock the Tauri invoke function
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

describe("FiscusApiClient", () => {
	let client: FiscusApiClient;

	beforeEach(() => {
		client = new FiscusApiClient();
		mockInvoke.mockClear();
	});

	describe("Authentication Methods", () => {
		describe("createUser", () => {
			it("should create a user successfully", async () => {
				const request: CreateUserRequest = {
					// file deepcode ignore NoHardcodedCredentials/test: <test>
					// file deepcode ignore NoHardcodedPasswords/test: <test>
					username: "testuser",
					email: "test@example.com",
					password: "StrongPass123!",
				};

				const expectedUser: User = {
					id: "123e4567-e89b-12d3-a456-426614174000",
					username: "testuser",
					email: "test@example.com",
					created_at: "2024-01-01T00:00:00Z",
					updated_at: "2024-01-01T00:00:00Z",
				};

				mockInvoke.mockResolvedValue(expectedUser);

				const result = await client.createUser(request);

				expect(mockInvoke).toHaveBeenCalledWith("create_user", { request });
				expect(result).toEqual(expectedUser);
			});

			it("should handle API errors", async () => {
				const request: CreateUserRequest = {
					username: "testuser",
					email: "test@example.com",
					password: "StrongPass123!",
				};

				const apiError = {
					type: "Validation",
					message: "Username already exists",
				};

				mockInvoke.mockRejectedValue(apiError);

				await expect(client.createUser(request)).rejects.toThrow(
					FiscusApiError,
				);
			});
		});

		describe("loginUser", () => {
			it("should login user successfully", async () => {
				const request: LoginRequest = {
					username: "testuser",
					password: "StrongPass123!",
				};

				const expectedResponse = {
					user: {
						id: "123e4567-e89b-12d3-a456-426614174000",
						username: "testuser",
						email: "test@example.com",
						created_at: "2024-01-01T00:00:00Z",
						updated_at: "2024-01-01T00:00:00Z",
					},
					session_token: "mock-session-token",
				};

				mockInvoke.mockResolvedValue(expectedResponse);

				const result = await client.loginUser(request);

				expect(mockInvoke).toHaveBeenCalledWith("login_user", { request });
				expect(result).toEqual(expectedResponse);
			});

			it("should handle login errors", async () => {
				const request: LoginRequest = {
					username: "testuser",
					password: "wrongpassword",
				};

				const apiError = {
					type: "Authentication",
					message: "Invalid credentials",
				};

				mockInvoke.mockRejectedValue(apiError);

				await expect(client.loginUser(request)).rejects.toThrow(FiscusApiError);
			});
		});

		describe("changePassword", () => {
			it("should change password successfully", async () => {
				const request = {
					user_id: "123e4567-e89b-12d3-a456-426614174000",
					current_password: "OldPass123!",
					new_password: "NewPass123!",
				};

				mockInvoke.mockResolvedValue(true);

				const result = await client.changePassword(request);

				expect(mockInvoke).toHaveBeenCalledWith("change_password", { request });
				expect(result).toBe(true);
			});
		});

		describe("getCurrentUser", () => {
			it("should get current user successfully", async () => {
				const userId = "123e4567-e89b-12d3-a456-426614174000";
				const expectedUser: User = {
					id: userId,
					username: "testuser",
					email: "test@example.com",
					created_at: "2024-01-01T00:00:00Z",
					updated_at: "2024-01-01T00:00:00Z",
				};

				mockInvoke.mockResolvedValue(expectedUser);

				const result = await client.getCurrentUser(userId);

				expect(mockInvoke).toHaveBeenCalledWith("get_current_user", { userId });
				expect(result).toEqual(expectedUser);
			});
		});
	});

	describe("Account Methods", () => {
		describe("createAccount", () => {
			it("should create an account successfully", async () => {
				const request = {
					user_id: "123e4567-e89b-12d3-a456-426614174000",
					account_type_id: "123e4567-e89b-12d3-a456-426614174001",
					name: "Checking Account",
					currency: "USD",
					balance: 1000.5,
				};

				const expectedAccount = {
					id: "123e4567-e89b-12d3-a456-426614174002",
					user_id: request.user_id,
					account_type_id: request.account_type_id,
					name: request.name,
					balance: request.balance,
					currency: request.currency,
					account_number: null,
					is_active: true,
					created_at: "2024-01-01T00:00:00Z",
					updated_at: "2024-01-01T00:00:00Z",
				};

				mockInvoke.mockResolvedValue(expectedAccount);

				const result = await client.createAccount(request);

				expect(mockInvoke).toHaveBeenCalledWith("create_account", { request });
				expect(result).toEqual(expectedAccount);
			});
		});

		describe("getAccounts", () => {
			it("should get accounts with filters", async () => {
				const filters = {
					user_id: "123e4567-e89b-12d3-a456-426614174000",
					is_active: true,
				};

				const expectedAccounts = [
					{
						id: "123e4567-e89b-12d3-a456-426614174002",
						user_id: filters.user_id,
						account_type_id: "123e4567-e89b-12d3-a456-426614174001",
						name: "Checking Account",
						balance: 1000.5,
						currency: "USD",
						account_number: null,
						is_active: true,
						created_at: "2024-01-01T00:00:00Z",
						updated_at: "2024-01-01T00:00:00Z",
					},
				];

				mockInvoke.mockResolvedValue(expectedAccounts);

				const result = await client.getAccounts(filters);

				expect(mockInvoke).toHaveBeenCalledWith("get_accounts", { filters });
				expect(result).toEqual(expectedAccounts);
			});
		});

		describe("updateAccount", () => {
			it("should update an account successfully", async () => {
				const accountId = "123e4567-e89b-12d3-a456-426614174002";
				const userId = "123e4567-e89b-12d3-a456-426614174000";
				const request = {
					name: "Updated Account Name",
					balance: 1500.75,
				};

				const expectedAccount = {
					id: accountId,
					user_id: userId,
					account_type_id: "123e4567-e89b-12d3-a456-426614174001",
					name: request.name,
					balance: request.balance,
					currency: "USD",
					account_number: null,
					is_active: true,
					created_at: "2024-01-01T00:00:00Z",
					updated_at: "2024-01-01T01:00:00Z",
				};

				mockInvoke.mockResolvedValue(expectedAccount);

				const result = await client.updateAccount(accountId, userId, request);

				expect(mockInvoke).toHaveBeenCalledWith("update_account", {
					accountId,
					userId,
					request,
				});
				expect(result).toEqual(expectedAccount);
			});
		});

		describe("deleteAccount", () => {
			it("should delete an account successfully", async () => {
				const accountId = "123e4567-e89b-12d3-a456-426614174002";
				const userId = "123e4567-e89b-12d3-a456-426614174000";

				mockInvoke.mockResolvedValue(true);

				const result = await client.deleteAccount(accountId, userId);

				expect(mockInvoke).toHaveBeenCalledWith("delete_account", {
					accountId,
					userId,
				});
				expect(result).toBe(true);
			});
		});

		describe("getAccountSummary", () => {
			it("should get account summary successfully", async () => {
				const userId = "123e4567-e89b-12d3-a456-426614174000";
				const expectedSummary = {
					total_assets: 5000.0,
					total_liabilities: 1000.0,
					net_worth: 4000.0,
					account_count: 3,
				};

				mockInvoke.mockResolvedValue(expectedSummary);

				const result = await client.getAccountSummary(userId);

				expect(mockInvoke).toHaveBeenCalledWith("get_account_summary", {
					userId,
				});
				expect(result).toEqual(expectedSummary);
			});
		});
	});

	describe("Error Handling", () => {
		it("should handle structured API errors", async () => {
			const apiError = {
				type: "Validation",
				message: "Invalid input data",
			};

			mockInvoke.mockRejectedValue(apiError);

			try {
				await client.getCurrentUser("invalid-id");
			} catch (error) {
				expect(error).toBeInstanceOf(FiscusApiError);
				expect((error as FiscusApiError).code).toBe("Validation");
				expect((error as FiscusApiError).message).toBe("Invalid input data");
			}
		});

		it("should handle string errors", async () => {
			const errorMessage = "Network error";

			mockInvoke.mockRejectedValue(errorMessage);

			try {
				await client.getCurrentUser("some-id");
			} catch (error) {
				expect(error).toBeInstanceOf(FiscusApiError);
				expect((error as FiscusApiError).code).toBe("UNKNOWN_ERROR");
				expect((error as FiscusApiError).message).toBe(errorMessage);
			}
		});

		it("should handle unknown errors", async () => {
			const unknownError = { someProperty: "value" };

			mockInvoke.mockRejectedValue(unknownError);

			try {
				await client.getCurrentUser("some-id");
			} catch (error) {
				expect(error).toBeInstanceOf(FiscusApiError);
				expect((error as FiscusApiError).code).toBe("UNKNOWN_ERROR");
				expect((error as FiscusApiError).message).toBe(
					"An unexpected error occurred",
				);
			}
		});
	});
});

describe("FiscusApiError", () => {
	it("should create error with message and code", () => {
		const error = new FiscusApiError("Test message", "TEST_CODE");

		expect(error.message).toBe("Test message");
		expect(error.code).toBe("TEST_CODE");
		expect(error.name).toBe("FiscusApiError");
	});

	it("should create error with status code", () => {
		const error = new FiscusApiError("Test message", "TEST_CODE", 400);

		expect(error.message).toBe("Test message");
		expect(error.code).toBe("TEST_CODE");
		expect(error.statusCode).toBe(400);
	});
});
