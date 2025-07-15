/**
 * Accounts store using Zustand
 * Manages account data and provides account-related actions
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	Account,
	AccountFilters,
	AccountSummaryResponse,
	CreateAccountRequest,
	UpdateAccountRequest,
} from "../types/api";

interface AccountsState {
	/** List of accounts */
	accounts: Account[];
	/** Currently selected account */
	selectedAccount: Account | null;
	/** Account summary data */
	summary: AccountSummaryResponse | null;
	/** Loading state for account operations */
	loading: boolean;
	/** Loading state for summary refresh operations */
	summaryLoading: boolean;
	/** Error state */
	error: FiscusApiError | null;
	/** Whether accounts have been loaded */
	initialized: boolean;
}

interface AccountsActions {
	/** Load accounts with optional filters */
	loadAccounts: (filters: AccountFilters) => Promise<void>;
	/** Create a new account */
	createAccount: (request: CreateAccountRequest) => Promise<Account | null>;
	/** Update an account */
	updateAccount: (
		accountId: string,
		userId: string,
		request: UpdateAccountRequest,
	) => Promise<Account | null>;
	/** Delete an account */
	deleteAccount: (accountId: string, userId: string) => Promise<boolean>;
	/** Load account summary */
	loadAccountSummary: (userId: string) => Promise<void>;
	/** Refresh summary after account operations (debounced) */
	refreshSummaryAfterOperation: (userId: string) => void;
	/** Select an account */
	selectAccount: (account: Account | null) => void;
	/** Get account by ID */
	getAccountById: (accountId: string) => Account | null;
	/** Refresh accounts data */
	refreshAccounts: (filters: AccountFilters) => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Reset store state */
	reset: () => void;
}

export type AccountsStore = AccountsState & AccountsActions;

// Timeout reference for debouncing summary refreshes
let summaryRefreshTimeout: NodeJS.Timeout | null = null;

export const useAccountsStore = create<AccountsStore>()((set, get) => ({
	// Initial state
	accounts: [],
	selectedAccount: null,
	summary: null,
	loading: false,
	summaryLoading: false,
	error: null,
	initialized: false,

	// Actions
	loadAccounts: async (filters: AccountFilters): Promise<void> => {
		set({ loading: true, error: null });

		try {
			const accounts = await apiClient.getAccounts(filters);

			set({
				accounts,
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
						: new FiscusApiError("Failed to load accounts", "INTERNAL_ERROR"),
			});
		}
	},

	createAccount: async (
		request: CreateAccountRequest,
	): Promise<Account | null> => {
		set({ loading: true, error: null });

		try {
			const newAccount = await apiClient.createAccount(request);

			// Add the new account to the list
			set((state) => ({
				accounts: [...state.accounts, newAccount],
				loading: false,
				error: null,
			}));

			// Refresh summary after account creation (debounced)
			get().refreshSummaryAfterOperation(request.user_id);

			return newAccount;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to create account", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	updateAccount: async (
		accountId: string,
		userId: string,
		request: UpdateAccountRequest,
	): Promise<Account | null> => {
		set({ loading: true, error: null });

		try {
			const updatedAccount = await apiClient.updateAccount(
				accountId,
				userId,
				request,
			);

			// Update the account in the list
			set((state) => ({
				accounts: state.accounts.map((account) =>
					account.id === accountId ? updatedAccount : account,
				),
				selectedAccount:
					state.selectedAccount?.id === accountId
						? updatedAccount
						: state.selectedAccount,
				loading: false,
				error: null,
			}));

			// Refresh summary after account update (debounced)
			get().refreshSummaryAfterOperation(userId);

			return updatedAccount;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to update account", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	deleteAccount: async (
		accountId: string,
		userId: string,
	): Promise<boolean> => {
		set({ loading: true, error: null });

		try {
			const success = await apiClient.deleteAccount(accountId, userId);

			if (success) {
				// Remove the account from the list
				set((state) => ({
					accounts: state.accounts.filter(
						(account) => account.id !== accountId,
					),
					selectedAccount:
						state.selectedAccount?.id === accountId
							? null
							: state.selectedAccount,
					loading: false,
					error: null,
				}));

				// Refresh summary after account deletion (debounced)
				get().refreshSummaryAfterOperation(userId);
			} else {
				set({
					loading: false,
					error: new FiscusApiError(
						"Failed to delete account",
						"OPERATION_NOT_ALLOWED",
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
						: new FiscusApiError("Failed to delete account", "INTERNAL_ERROR"),
			});

			return false;
		}
	},

	loadAccountSummary: async (userId: string): Promise<void> => {
		try {
			const summary = await apiClient.getAccountSummary(userId);

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
								"Failed to load account summary",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	refreshSummaryAfterOperation: (userId: string) => {
		// Clear any existing timeout to debounce rapid successive calls
		if (summaryRefreshTimeout) {
			clearTimeout(summaryRefreshTimeout);
		}

		// Only refresh if summary exists (has been loaded before)
		if (!get().summary) return;

		// Debounce the refresh to prevent race conditions
		summaryRefreshTimeout = setTimeout(async () => {
			set({ summaryLoading: true });
			try {
				await get().loadAccountSummary(userId);
			} catch (error) {
				// Log the error but don't set it in the store as this is a background operation
				// and shouldn't affect the main operation's success
				console.warn(
					"Failed to refresh summary after account operation:",
					error,
				);
			} finally {
				set({ summaryLoading: false });
				summaryRefreshTimeout = null;
			}
		}, 300); // 300ms debounce delay
	},

	selectAccount: (account: Account | null) => {
		set({ selectedAccount: account });
	},

	getAccountById: (accountId: string): Account | null => {
		const { accounts } = get();
		return accounts.find((account) => account.id === accountId) || null;
	},

	refreshAccounts: async (filters: AccountFilters): Promise<void> => {
		await get().loadAccounts(filters);
	},

	clearError: () => {
		set({ error: null });
	},

	setLoading: (loading: boolean) => {
		set({ loading });
	},

	reset: () => {
		// Clear any pending summary refresh timeout
		if (summaryRefreshTimeout) {
			clearTimeout(summaryRefreshTimeout);
			summaryRefreshTimeout = null;
		}

		set({
			accounts: [],
			selectedAccount: null,
			summary: null,
			loading: false,
			summaryLoading: false,
			error: null,
			initialized: false,
		});
	},
}));

/**
 * Selector hooks for common account state
 */
export const useAccounts = () => {
	const { accounts, loading, error } = useAccountsStore();
	return { accounts, loading, error };
};

export const useAccountsActions = () => {
	const {
		loadAccounts,
		createAccount,
		updateAccount,
		deleteAccount,
		refreshAccounts,
		clearError,
	} = useAccountsStore();
	return {
		loadAccounts,
		createAccount,
		updateAccount,
		deleteAccount,
		refreshAccounts,
		clearError,
	};
};

export const useSelectedAccount = () => {
	const { selectedAccount, selectAccount } = useAccountsStore();
	return { selectedAccount, selectAccount };
};

export const useAccountSummary = () => {
	const { summary, summaryLoading, loadAccountSummary } = useAccountsStore();
	return { summary, summaryLoading, loadAccountSummary };
};

/**
 * Hook to get accounts by type
 */
export const useAccountsByType = (accountTypeId?: string) => {
	const accounts = useAccountsStore((state) => state.accounts);

	if (!accountTypeId) return accounts;

	return accounts.filter(
		(account) => account.account_type_id === accountTypeId,
	);
};

/**
 * Hook to get active accounts only
 */
export const useActiveAccounts = () => {
	const accounts = useAccountsStore((state) => state.accounts);
	return accounts.filter((account) => account.is_active);
};

/**
 * Hook to get account by ID with reactive updates
 */
export const useAccountById = (accountId: string) => {
	return useAccountsStore(
		(state) =>
			state.accounts.find((account) => account.id === accountId) || null,
	);
};
