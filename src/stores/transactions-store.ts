/**
 * Transactions store using Zustand
 * Manages transaction data and provides transaction-related actions
 */

import { create } from "zustand";
import { apiClient, FiscusApiError } from "../api/client";
import type {
	BulkTransactionRequest,
	CreateTransactionRequest,
	CreateTransferRequest,
	Transaction,
	TransactionFilters,
	TransactionStatsResponse,
	TransactionSummaryResponse,
	TransactionType,
	Transfer,
	UpdateTransactionRequest,
} from "../types/api";

interface TransactionsState {
	/** List of transactions */
	transactions: Transaction[];
	/** Currently selected transaction */
	selectedTransaction: Transaction | null;
	/** Transaction summary data */
	summary: TransactionSummaryResponse | null;
	/** Transaction statistics */
	stats: TransactionStatsResponse | null;
	/** Recent transfers */
	transfers: Transfer[];
	/** Pagination data */
	pagination: {
		total: number;
		page: number;
		per_page: number;
		total_pages: number;
	} | null;
	/** Selected transaction IDs for bulk operations */
	selectedTransactionIds: string[];
	/** Loading state for transaction operations */
	loading: boolean;
	/** Loading state for specific operations */
	loadingStates: {
		transactions: boolean;
		stats: boolean;
		bulk: boolean;
		export: boolean;
	};
	/** Error state */
	error: FiscusApiError | null;
	/** Whether transactions have been loaded */
	initialized: boolean;
	/** Current filter settings */
	currentFilters: TransactionFilters | null;
	/** Search query */
	searchQuery: string;
	/** Sort configuration */
	sortConfig: {
		field: string;
		direction: "asc" | "desc";
	};
}

interface TransactionsActions {
	/** Load transactions with filters */
	loadTransactions: (filters: TransactionFilters) => Promise<void>;
	/** Create a new transaction */
	createTransaction: (
		request: CreateTransactionRequest,
	) => Promise<Transaction | null>;
	/** Update a transaction */
	updateTransaction: (
		transactionId: string,
		userId: string,
		request: UpdateTransactionRequest,
	) => Promise<Transaction | null>;
	/** Delete a transaction */
	deleteTransaction: (
		transactionId: string,
		userId: string,
	) => Promise<boolean>;
	/** Create a transfer between accounts */
	createTransfer: (request: CreateTransferRequest) => Promise<Transfer | null>;
	/** Load transaction summary */
	loadTransactionSummary: (
		userId: string,
		startDate?: string,
		endDate?: string,
	) => Promise<void>;
	/** Select a transaction */
	selectTransaction: (transaction: Transaction | null) => void;
	/** Get transaction by ID */
	getTransactionById: (transactionId: string) => Transaction | null;
	/** Refresh transactions data */
	refreshTransactions: () => Promise<void>;
	/** Clear error state */
	clearError: () => void;
	/** Set loading state */
	setLoading: (loading: boolean) => void;
	/** Reset store state */
	reset: () => void;
	/** Load transactions with pagination */
	loadTransactionsPaginated: (filters: TransactionFilters) => Promise<void>;
	/** Load transaction statistics */
	loadStats: (filters: TransactionFilters) => Promise<void>;
	/** Bulk operations on transactions */
	bulkOperations: (request: BulkTransactionRequest) => Promise<string | null>;
	/** Toggle transaction selection for bulk operations */
	toggleTransactionSelection: (transactionId: string) => void;
	/** Select all visible transactions */
	selectAllTransactions: () => void;
	/** Clear all transaction selections */
	clearTransactionSelection: () => void;
	/** Set search query */
	setSearchQuery: (query: string) => void;
	/** Set sort configuration */
	setSortConfig: (field: string, direction: "asc" | "desc") => void;
	/** Apply filters and reload transactions */
	applyFilters: (filters: Partial<TransactionFilters>) => Promise<void>;
}

export type TransactionsStore = TransactionsState & TransactionsActions;

export const useTransactionsStore = create<TransactionsStore>()((set, get) => ({
	// Initial state
	transactions: [],
	selectedTransaction: null,
	summary: null,
	stats: null,
	transfers: [],
	pagination: null,
	selectedTransactionIds: [],
	loading: false,
	loadingStates: {
		transactions: false,
		stats: false,
		bulk: false,
		export: false,
	},
	error: null,
	initialized: false,
	currentFilters: null,
	searchQuery: "",
	sortConfig: {
		field: "transaction_date",
		direction: "desc",
	},

	// Actions
	loadTransactions: async (filters: TransactionFilters): Promise<void> => {
		set({ loading: true, error: null, currentFilters: filters });

		try {
			const transactions = await apiClient.getTransactions(filters);

			set({
				transactions,
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
								"Failed to load transactions",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	createTransaction: async (
		request: CreateTransactionRequest,
	): Promise<Transaction | null> => {
		set({ loading: true, error: null });

		try {
			const newTransaction = await apiClient.createTransaction(request);

			// Add the new transaction to the list (at the beginning for chronological order)
			set((state) => ({
				transactions: [newTransaction, ...state.transactions],
				loading: false,
				error: null,
			}));

			// Refresh summary if it exists
			if (get().summary) {
				get().loadTransactionSummary(request.user_id);
			}

			return newTransaction;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to create transaction",
								"INTERNAL_ERROR",
							),
			});

			return null;
		}
	},

	updateTransaction: async (
		transactionId: string,
		userId: string,
		request: UpdateTransactionRequest,
	): Promise<Transaction | null> => {
		set({ loading: true, error: null });

		try {
			const updatedTransaction = await apiClient.updateTransaction(
				transactionId,
				userId,
				request,
			);

			// Update the transaction in the list
			set((state) => ({
				transactions: state.transactions.map((transaction) =>
					transaction.id === transactionId ? updatedTransaction : transaction,
				),
				selectedTransaction:
					state.selectedTransaction?.id === transactionId
						? updatedTransaction
						: state.selectedTransaction,
				loading: false,
				error: null,
			}));

			// Refresh summary if it exists
			if (get().summary) {
				get().loadTransactionSummary(userId);
			}

			return updatedTransaction;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to update transaction",
								"INTERNAL_ERROR",
							),
			});

			return null;
		}
	},

	deleteTransaction: async (
		transactionId: string,
		userId: string,
	): Promise<boolean> => {
		set({ loading: true, error: null });

		try {
			const success = await apiClient.deleteTransaction(transactionId, userId);

			if (success) {
				// Remove the transaction from the list
				set((state) => ({
					transactions: state.transactions.filter(
						(transaction) => transaction.id !== transactionId,
					),
					selectedTransaction:
						state.selectedTransaction?.id === transactionId
							? null
							: state.selectedTransaction,
					loading: false,
					error: null,
				}));

				// Refresh summary if it exists
				if (get().summary) {
					get().loadTransactionSummary(userId);
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
						: new FiscusApiError(
								"Failed to delete transaction",
								"INTERNAL_ERROR",
							),
			});

			return false;
		}
	},

	createTransfer: async (
		request: CreateTransferRequest,
	): Promise<Transfer | null> => {
		set({ loading: true, error: null });

		try {
			const newTransfer = await apiClient.createTransfer(request);

			// Add the transfer to the list
			set((state) => ({
				transfers: [newTransfer, ...state.transfers],
				loading: false,
				error: null,
			}));

			// Refresh transactions if current filters match
			const { currentFilters } = get();
			if (currentFilters) {
				get().refreshTransactions();
			}

			// Refresh summary if it exists
			if (get().summary) {
				get().loadTransactionSummary(request.user_id);
			}

			return newTransfer;
		} catch (error) {
			set({
				loading: false,
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError("Failed to create transfer", "INTERNAL_ERROR"),
			});

			return null;
		}
	},

	loadTransactionSummary: async (
		userId: string,
		startDate?: string,
		endDate?: string,
	): Promise<void> => {
		try {
			const summary = await apiClient.getTransactionSummary(
				userId,
				startDate,
				endDate,
			);

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
								"Failed to load transaction summary",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	selectTransaction: (transaction: Transaction | null) => {
		set({ selectedTransaction: transaction });
	},

	getTransactionById: (transactionId: string): Transaction | null => {
		const { transactions } = get();
		return (
			transactions.find((transaction) => transaction.id === transactionId) ||
			null
		);
	},

	refreshTransactions: async (): Promise<void> => {
		const { currentFilters } = get();
		if (currentFilters) {
			await get().loadTransactions(currentFilters);
		}
	},

	clearError: () => {
		set({ error: null });
	},

	setLoading: (loading: boolean) => {
		set({ loading });
	},

	reset: () => {
		set({
			transactions: [],
			selectedTransaction: null,
			summary: null,
			stats: null,
			transfers: [],
			pagination: null,
			selectedTransactionIds: [],
			loading: false,
			loadingStates: {
				transactions: false,
				stats: false,
				bulk: false,
				export: false,
			},
			error: null,
			initialized: false,
			currentFilters: null,
			searchQuery: "",
			sortConfig: {
				field: "transaction_date",
				direction: "desc",
			},
		});
	},

	// New enhanced actions
	loadTransactionsPaginated: async (
		filters: TransactionFilters,
	): Promise<void> => {
		set((state) => ({
			...state,
			loadingStates: { ...state.loadingStates, transactions: true },
			error: null,
		}));

		try {
			const response = await apiClient.getTransactionsPaginated(filters);

			set({
				transactions: response.data,
				pagination: {
					total: response.total,
					page: response.page,
					per_page: response.per_page,
					total_pages: response.total_pages,
				},
				loadingStates: { ...get().loadingStates, transactions: false },
				error: null,
				initialized: true,
				currentFilters: filters,
			});
		} catch (error) {
			set({
				loadingStates: { ...get().loadingStates, transactions: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load transactions",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	loadStats: async (filters: TransactionFilters): Promise<void> => {
		set((state) => ({
			...state,
			loadingStates: { ...state.loadingStates, stats: true },
			error: null,
		}));

		try {
			const stats = await apiClient.getTransactionStats(filters);

			set({
				stats,
				loadingStates: { ...get().loadingStates, stats: false },
				error: null,
			});
		} catch (error) {
			set({
				loadingStates: { ...get().loadingStates, stats: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to load transaction statistics",
								"INTERNAL_ERROR",
							),
			});
		}
	},

	bulkOperations: async (
		request: BulkTransactionRequest,
	): Promise<string | null> => {
		set((state) => ({
			...state,
			loadingStates: { ...state.loadingStates, bulk: true },
			error: null,
		}));

		try {
			const result = await apiClient.bulkTransactionOperations(request);

			set({
				loadingStates: { ...get().loadingStates, bulk: false },
				error: null,
				selectedTransactionIds: [], // Clear selection after operation
			});

			// Refresh transactions after bulk operation
			const { currentFilters } = get();
			if (currentFilters) {
				await get().loadTransactions(currentFilters);
			}

			return result;
		} catch (error) {
			set({
				loadingStates: { ...get().loadingStates, bulk: false },
				error:
					error instanceof FiscusApiError
						? error
						: new FiscusApiError(
								"Failed to perform bulk operation",
								"INTERNAL_ERROR",
							),
			});
			return null;
		}
	},

	toggleTransactionSelection: (transactionId: string): void => {
		set((state) => {
			const isSelected = state.selectedTransactionIds.includes(transactionId);
			const newSelection = isSelected
				? state.selectedTransactionIds.filter((id) => id !== transactionId)
				: [...state.selectedTransactionIds, transactionId];

			return {
				...state,
				selectedTransactionIds: newSelection,
			};
		});
	},

	selectAllTransactions: (): void => {
		set((state) => ({
			...state,
			selectedTransactionIds: state.transactions.map((t) => t.id),
		}));
	},

	clearTransactionSelection: (): void => {
		set((state) => ({
			...state,
			selectedTransactionIds: [],
		}));
	},

	setSearchQuery: (query: string): void => {
		set((state) => ({
			...state,
			searchQuery: query,
		}));
	},

	setSortConfig: (field: string, direction: "asc" | "desc"): void => {
		set((state) => ({
			...state,
			sortConfig: { field, direction },
		}));
	},

	applyFilters: async (filters: Partial<TransactionFilters>): Promise<void> => {
		const { currentFilters } = get();
		const newFilters = { ...currentFilters, ...filters } as TransactionFilters;

		set((state) => ({
			...state,
			currentFilters: newFilters,
		}));

		await get().loadTransactions(newFilters);
	},
}));

/**
 * Selector hooks for common transaction state
 */
export const useTransactions = () => {
	const { transactions, loading, error } = useTransactionsStore();
	return { transactions, loading, error };
};

export const useTransactionsActions = () => {
	const {
		loadTransactions,
		createTransaction,
		updateTransaction,
		deleteTransaction,
		createTransfer,
		refreshTransactions,
		clearError,
	} = useTransactionsStore();
	return {
		loadTransactions,
		createTransaction,
		updateTransaction,
		deleteTransaction,
		createTransfer,
		refreshTransactions,
		clearError,
	};
};

export const useSelectedTransaction = () => {
	const { selectedTransaction, selectTransaction } = useTransactionsStore();
	return { selectedTransaction, selectTransaction };
};

export const useTransactionSummary = () => {
	const { summary, loadTransactionSummary } = useTransactionsStore();
	return { summary, loadTransactionSummary };
};

/**
 * Hook to get transactions by type
 */
export const useTransactionsByType = (type: TransactionType) => {
	const transactions = useTransactionsStore((state) => state.transactions);
	return transactions.filter(
		(transaction) => transaction.transaction_type === type,
	);
};

/**
 * Hook to get transactions by account
 */
export const useTransactionsByAccount = (accountId: string) => {
	const transactions = useTransactionsStore((state) => state.transactions);
	return transactions.filter(
		(transaction) => transaction.account_id === accountId,
	);
};

/**
 * Hook to get recent transactions (last 10)
 */
export const useRecentTransactions = (limit: number = 10) => {
	const transactions = useTransactionsStore((state) => state.transactions);
	return transactions
		.sort(
			(a, b) =>
				new Date(b.transaction_date).getTime() -
				new Date(a.transaction_date).getTime(),
		)
		.slice(0, limit);
};

/**
 * Hook to get transaction by ID with reactive updates
 */
export const useTransactionById = (transactionId: string) => {
	return useTransactionsStore(
		(state) =>
			state.transactions.find(
				(transaction) => transaction.id === transactionId,
			) || null,
	);
};
