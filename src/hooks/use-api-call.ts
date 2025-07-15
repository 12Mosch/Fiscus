/**
 * Custom hook for API calls with loading states and error handling
 * Provides a consistent interface for all API operations
 */

import { useCallback, useEffect, useState } from "react";
import { FiscusApiError } from "../api/client";

/**
 * State interface for API calls
 */
export interface ApiCallState<T> {
	/** The data returned from the API call */
	data: T | null;
	/** Loading state */
	loading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether the call has been executed */
	called: boolean;
}

/**
 * Options for the useApiCall hook
 */
export interface UseApiCallOptions {
	/** Whether to execute the API call immediately on mount */
	immediate?: boolean;
	/** Callback to execute on success */
	onSuccess?: (data: unknown) => void;
	/** Callback to execute on error */
	onError?: (error: FiscusApiError) => void;
	/** Callback to execute when loading state changes */
	onLoadingChange?: (loading: boolean) => void;
}

/**
 * Custom hook for handling API calls with loading states and error handling
 * @param apiCall The API function to call
 * @param options Configuration options
 * @returns Object containing data, loading state, error, and execute function
 */
export function useApiCall<T>(
	apiCall: () => Promise<T>,
	options: UseApiCallOptions = {},
) {
	const { immediate = false, onSuccess, onError, onLoadingChange } = options;

	const [state, setState] = useState<ApiCallState<T>>({
		data: null,
		loading: false,
		error: null,
		called: false,
	});

	const execute = useCallback(async (): Promise<T | null> => {
		setState((prev) => ({ ...prev, loading: true, error: null, called: true }));
		onLoadingChange?.(true);

		try {
			const result = await apiCall();
			setState((prev) => ({ ...prev, data: result, loading: false }));
			onLoadingChange?.(false);
			onSuccess?.(result);
			return result;
		} catch (err) {
			const error =
				err instanceof FiscusApiError
					? err
					: new FiscusApiError(
							err instanceof Error
								? err.message
								: typeof err === "string"
									? err
									: "An unexpected error occurred",
							"UNKNOWN_ERROR",
						);
			setState((prev) => ({ ...prev, error, loading: false }));
			onLoadingChange?.(false);
			onError?.(error);
			return null;
		}
	}, [apiCall, onSuccess, onError, onLoadingChange]);

	const reset = useCallback(() => {
		setState({
			data: null,
			loading: false,
			error: null,
			called: false,
		});
	}, []);

	// Execute immediately if requested
	useEffect(() => {
		if (immediate) {
			execute();
		}
	}, [immediate, execute]);

	return {
		...state,
		execute,
		reset,
	};
}

/**
 * Custom hook for API calls with parameters
 * @param apiCall The API function to call (receives parameters)
 * @param options Configuration options
 * @returns Object containing data, loading state, error, and execute function
 */
export function useApiCallWithParams<T, P extends unknown[]>(
	apiCall: (...params: P) => Promise<T>,
	options: UseApiCallOptions = {},
) {
	const { onSuccess, onError, onLoadingChange } = options;

	const [state, setState] = useState<ApiCallState<T>>({
		data: null,
		loading: false,
		error: null,
		called: false,
	});

	const execute = useCallback(
		async (...params: P): Promise<T | null> => {
			setState((prev) => ({
				...prev,
				loading: true,
				error: null,
				called: true,
			}));
			onLoadingChange?.(true);

			try {
				const result = await apiCall(...params);
				setState((prev) => ({ ...prev, data: result, loading: false }));
				onLoadingChange?.(false);
				onSuccess?.(result);
				return result;
			} catch (err) {
				const error =
					err instanceof FiscusApiError
						? err
						: new FiscusApiError(
								err instanceof Error
									? err.message
									: typeof err === "string"
										? err
										: "An unexpected error occurred",
								"UNKNOWN_ERROR",
							);
				setState((prev) => ({ ...prev, error, loading: false }));
				onLoadingChange?.(false);
				onError?.(error);
				return null;
			}
		},
		[apiCall, onSuccess, onError, onLoadingChange],
	);

	const reset = useCallback(() => {
		setState({
			data: null,
			loading: false,
			error: null,
			called: false,
		});
	}, []);

	return {
		...state,
		execute,
		reset,
	};
}

/**
 * Custom hook for mutation operations (create, update, delete)
 * Provides optimistic updates and rollback functionality
 */
export function useMutation<T, P extends unknown[]>(
	mutationFn: (...params: P) => Promise<T>,
	options: UseApiCallOptions & {
		/** Function to perform optimistic update */
		optimisticUpdate?: (...params: P) => void;
		/** Function to rollback optimistic update on error */
		rollback?: (...params: P) => void;
	} = {},
) {
	const { optimisticUpdate, rollback, onSuccess, onError, onLoadingChange } =
		options;

	const [state, setState] = useState<ApiCallState<T>>({
		data: null,
		loading: false,
		error: null,
		called: false,
	});

	const mutate = useCallback(
		async (...params: P): Promise<T | null> => {
			setState((prev) => ({
				...prev,
				loading: true,
				error: null,
				called: true,
			}));
			onLoadingChange?.(true);

			// Perform optimistic update
			optimisticUpdate?.(...params);

			try {
				const result = await mutationFn(...params);
				setState((prev) => ({ ...prev, data: result, loading: false }));
				onLoadingChange?.(false);
				onSuccess?.(result);
				return result;
			} catch (err) {
				// Rollback optimistic update on error
				rollback?.(...params);

				const error =
					err instanceof FiscusApiError
						? err
						: new FiscusApiError(
								err instanceof Error
									? err.message
									: typeof err === "string"
										? err
										: "An unexpected error occurred",
								"UNKNOWN_ERROR",
							);
				setState((prev) => ({ ...prev, error, loading: false }));
				onLoadingChange?.(false);
				onError?.(error);
				return null;
			}
		},
		[
			mutationFn,
			optimisticUpdate,
			rollback,
			onSuccess,
			onError,
			onLoadingChange,
		],
	);

	const reset = useCallback(() => {
		setState({
			data: null,
			loading: false,
			error: null,
			called: false,
		});
	}, []);

	return {
		...state,
		mutate,
		reset,
	};
}

/**
 * Utility hook for handling multiple API calls
 * Useful for loading related data in parallel
 */
export function useMultipleApiCalls<T extends Record<string, unknown>>(
	apiCalls: { [K in keyof T]: () => Promise<T[K]> },
	options: UseApiCallOptions = {},
) {
	const { immediate = false, onSuccess, onError, onLoadingChange } = options;

	const [state, setState] = useState<{
		data: Partial<T>;
		loading: boolean;
		error: FiscusApiError | null;
		called: boolean;
	}>({
		data: {},
		loading: false,
		error: null,
		called: false,
	});

	const execute = useCallback(async (): Promise<Partial<T>> => {
		setState((prev) => ({ ...prev, loading: true, error: null, called: true }));
		onLoadingChange?.(true);

		try {
			const promises = Object.entries(apiCalls).map(async ([key, apiCall]) => {
				const result = await (apiCall as () => Promise<unknown>)();
				return [key, result] as const;
			});

			const results = await Promise.all(promises);
			const data = Object.fromEntries(results) as T;

			setState((prev) => ({ ...prev, data, loading: false }));
			onLoadingChange?.(false);
			onSuccess?.(data);
			return data;
		} catch (err) {
			const error =
				err instanceof FiscusApiError
					? err
					: new FiscusApiError(
							err instanceof Error
								? err.message
								: typeof err === "string"
									? err
									: "An unexpected error occurred",
							"UNKNOWN_ERROR",
						);
			setState((prev) => ({ ...prev, error, loading: false }));
			onLoadingChange?.(false);
			onError?.(error);
			return {};
		}
	}, [apiCalls, onSuccess, onError, onLoadingChange]);

	const reset = useCallback(() => {
		setState({
			data: {},
			loading: false,
			error: null,
			called: false,
		});
	}, []);

	// Execute immediately if requested
	useEffect(() => {
		if (immediate) {
			execute();
		}
	}, [immediate, execute]);

	return {
		...state,
		execute,
		reset,
	};
}
