/**
 * Tests for useApiCall hooks
 */

import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { FiscusApiError } from "../../api/client";
import { useApiCall, useApiCallWithParams, useMutation } from "../use-api-call";

describe("useApiCall", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe("Basic functionality", () => {
		it("should initialize with correct default state", () => {
			const mockApiCall = vi.fn().mockResolvedValue("test data");
			const { result } = renderHook(() => useApiCall(mockApiCall));

			expect(result.current.data).toBeNull();
			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
			expect(result.current.called).toBe(false);
		});

		it("should execute API call successfully", async () => {
			const mockData = { id: 1, name: "Test" };
			const mockApiCall = vi.fn().mockResolvedValue(mockData);
			const { result } = renderHook(() => useApiCall(mockApiCall));

			await act(async () => {
				await result.current.execute();
			});

			expect(result.current.data).toEqual(mockData);
			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
			expect(result.current.called).toBe(true);
		});

		it("should handle API call errors", async () => {
			const mockError = new FiscusApiError("Test error", "TEST_ERROR");
			const mockApiCall = vi.fn().mockRejectedValue(mockError);
			const { result } = renderHook(() => useApiCall(mockApiCall));

			await act(async () => {
				await result.current.execute();
			});

			expect(result.current.data).toBeNull();
			expect(result.current.loading).toBe(false);
			expect(result.current.error).toEqual(mockError);
			expect(result.current.called).toBe(true);
		});

		it("should set loading state during execution", async () => {
			let resolveApiCall!: (value: string) => void;
			const apiCallPromise = new Promise<string>((resolve) => {
				resolveApiCall = resolve;
			});
			const mockApiCall = vi.fn().mockReturnValue(apiCallPromise);

			const { result } = renderHook(() => useApiCall(mockApiCall));

			// Start execution
			act(() => {
				result.current.execute();
			});

			// Check loading state
			expect(result.current.loading).toBe(true);
			expect(result.current.called).toBe(true);

			// Resolve the API call
			act(() => {
				resolveApiCall("test data");
			});

			await waitFor(() => {
				expect(result.current.loading).toBe(false);
			});

			expect(result.current.data).toBe("test data");
		});

		it("should reset state correctly", async () => {
			const mockData = { id: 1, name: "Test" };
			const mockApiCall = vi.fn().mockResolvedValue(mockData);
			const { result } = renderHook(() => useApiCall(mockApiCall));

			// Execute and get data
			await act(async () => {
				await result.current.execute();
			});

			expect(result.current.data).toEqual(mockData);
			expect(result.current.called).toBe(true);

			// Reset
			act(() => {
				result.current.reset();
			});

			expect(result.current.data).toBeNull();
			expect(result.current.loading).toBe(false);
			expect(result.current.error).toBeNull();
			expect(result.current.called).toBe(false);
		});
	});

	describe("Options", () => {
		it("should execute immediately when immediate option is true", async () => {
			const mockData = { id: 1, name: "Test" };
			const mockApiCall = vi.fn().mockResolvedValue(mockData);

			const { result } = renderHook(() =>
				useApiCall(mockApiCall, { immediate: true }),
			);

			await waitFor(() => {
				expect(result.current.data).toEqual(mockData);
			});

			expect(mockApiCall).toHaveBeenCalledTimes(1);
			expect(result.current.called).toBe(true);
		});

		it("should call onSuccess callback", async () => {
			const mockData = { id: 1, name: "Test" };
			const mockApiCall = vi.fn().mockResolvedValue(mockData);
			const onSuccess = vi.fn();

			const { result } = renderHook(() =>
				useApiCall(mockApiCall, { onSuccess }),
			);

			await act(async () => {
				await result.current.execute();
			});

			expect(onSuccess).toHaveBeenCalledWith(mockData);
		});

		it("should call onError callback", async () => {
			const mockError = new FiscusApiError("Test error", "TEST_ERROR");
			const mockApiCall = vi.fn().mockRejectedValue(mockError);
			const onError = vi.fn();

			const { result } = renderHook(() => useApiCall(mockApiCall, { onError }));

			await act(async () => {
				await result.current.execute();
			});

			expect(onError).toHaveBeenCalledWith(mockError);
		});

		it("should call onLoadingChange callback", async () => {
			const mockData = { id: 1, name: "Test" };
			const mockApiCall = vi.fn().mockResolvedValue(mockData);
			const onLoadingChange = vi.fn();

			const { result } = renderHook(() =>
				useApiCall(mockApiCall, { onLoadingChange }),
			);

			await act(async () => {
				await result.current.execute();
			});

			expect(onLoadingChange).toHaveBeenCalledWith(true);
			expect(onLoadingChange).toHaveBeenCalledWith(false);
		});
	});
});

describe("useApiCallWithParams", () => {
	it("should execute API call with parameters", async () => {
		const mockData = { id: 1, name: "Test" };
		const mockApiCall = vi.fn().mockResolvedValue(mockData);
		const { result } = renderHook(() => useApiCallWithParams(mockApiCall));

		const params = ["param1", "param2"];
		await act(async () => {
			await result.current.execute(...params);
		});

		expect(mockApiCall).toHaveBeenCalledWith(...params);
		expect(result.current.data).toEqual(mockData);
	});

	it("should handle different parameter types", async () => {
		const mockData = { success: true };
		const mockApiCall = vi.fn().mockResolvedValue(mockData);
		const { result } = renderHook(() => useApiCallWithParams(mockApiCall));

		const params = [
			"string-param",
			123,
			{ object: "param" },
			["array", "param"],
		];

		await act(async () => {
			await result.current.execute(...params);
		});

		expect(mockApiCall).toHaveBeenCalledWith(...params);
		expect(result.current.data).toEqual(mockData);
	});
});

describe("useMutation", () => {
	it("should execute mutation successfully", async () => {
		const mockData = { id: 1, name: "Created" };
		const mockMutation = vi.fn().mockResolvedValue(mockData);
		const { result } = renderHook(() => useMutation(mockMutation));

		const params = ["param1", "param2"];
		await act(async () => {
			await result.current.mutate(...params);
		});

		expect(mockMutation).toHaveBeenCalledWith(...params);
		expect(result.current.data).toEqual(mockData);
	});

	it("should handle optimistic updates", async () => {
		const mockData = { id: 1, name: "Updated" };
		const mockMutation = vi.fn().mockResolvedValue(mockData);
		const optimisticUpdate = vi.fn();
		const rollback = vi.fn();

		const { result } = renderHook(() =>
			useMutation(mockMutation, { optimisticUpdate, rollback }),
		);

		const params = ["param1", "param2"];
		await act(async () => {
			await result.current.mutate(...params);
		});

		expect(optimisticUpdate).toHaveBeenCalledWith(...params);
		expect(rollback).not.toHaveBeenCalled();
		expect(result.current.data).toEqual(mockData);
	});

	it("should rollback on error", async () => {
		const mockError = new FiscusApiError("Mutation failed", "MUTATION_ERROR");
		const mockMutation = vi.fn().mockRejectedValue(mockError);
		const optimisticUpdate = vi.fn();
		const rollback = vi.fn();

		const { result } = renderHook(() =>
			useMutation(mockMutation, { optimisticUpdate, rollback }),
		);

		const params = ["param1", "param2"];
		await act(async () => {
			await result.current.mutate(...params);
		});

		expect(optimisticUpdate).toHaveBeenCalledWith(...params);
		expect(rollback).toHaveBeenCalledWith(...params);
		expect(result.current.error).toEqual(mockError);
	});
});

describe("Error handling", () => {
	it("should convert non-FiscusApiError to FiscusApiError", async () => {
		const mockError = new Error("Generic error");
		const mockApiCall = vi.fn().mockRejectedValue(mockError);
		const { result } = renderHook(() => useApiCall(mockApiCall));

		await act(async () => {
			await result.current.execute();
		});

		expect(result.current.error).toBeInstanceOf(FiscusApiError);
		expect(result.current.error?.message).toBe("Generic error");
		expect(result.current.error?.code).toBe("UNKNOWN_ERROR");
	});

	it("should handle string errors", async () => {
		const mockApiCall = vi.fn().mockRejectedValue("String error");
		const { result } = renderHook(() => useApiCall(mockApiCall));

		await act(async () => {
			await result.current.execute();
		});

		expect(result.current.error).toBeInstanceOf(FiscusApiError);
		expect(result.current.error?.message).toBe("String error");
		expect(result.current.error?.code).toBe("UNKNOWN_ERROR");
	});

	it("should handle unknown error types", async () => {
		const mockApiCall = vi.fn().mockRejectedValue({ unknown: "error" });
		const { result } = renderHook(() => useApiCall(mockApiCall));

		await act(async () => {
			await result.current.execute();
		});

		expect(result.current.error).toBeInstanceOf(FiscusApiError);
		expect(result.current.error?.message).toBe("An unexpected error occurred");
		expect(result.current.error?.code).toBe("UNKNOWN_ERROR");
	});
});
