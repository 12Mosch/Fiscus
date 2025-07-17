/**
 * Transaction List Component
 * Displays transactions in a responsive table with sorting, filtering, and selection
 */

import {
	ChevronDown,
	ChevronUp,
	Download,
	Edit,
	Eye,
	MoreHorizontal,
	Search,
	Trash2,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useUserId } from "@/stores/auth-store";
import { useTransactionsStore } from "@/stores/transactions-store";
import type { Transaction, TransactionFilters } from "@/types/api";
import { formatCurrency } from "@/utils/currency";
import { formatDate } from "@/utils/date";

interface TransactionListProps {
	/** Optional filters to apply */
	initialFilters?: Partial<TransactionFilters>;
	/** Whether to show bulk actions */
	showBulkActions?: boolean;
	/** Whether to show pagination */
	showPagination?: boolean;
	/** Number of items per page */
	pageSize?: number;
	/** Callback when transaction is selected for editing */
	onEditTransaction?: (transaction: Transaction) => void;
	/** Callback when transaction is selected for viewing */
	onViewTransaction?: (transaction: Transaction) => void;
}

export function TransactionList({
	initialFilters,
	showBulkActions = true,
	showPagination = true,
	pageSize = 50,
	onEditTransaction,
	onViewTransaction,
}: TransactionListProps) {
	const userId = useUserId();
	const {
		transactions,
		pagination,
		selectedTransactionIds,
		loadingStates,
		error,
		searchQuery,
		sortConfig,
		currentFilters,
		loadTransactionsPaginated,
		toggleTransactionSelection,
		selectAllTransactions,
		clearTransactionSelection,
		setSearchQuery,
		setSortConfig,
		applyFilters,
	} = useTransactionsStore();

	const [localSearchQuery, setLocalSearchQuery] = useState(searchQuery);

	// Load transactions on mount and when filters change
	useEffect(() => {
		if (userId) {
			const filters: TransactionFilters = {
				user_id: userId,
				limit: pageSize,
				offset: 0,
				search: searchQuery || undefined,
				sort_by: sortConfig.field,
				sort_direction: sortConfig.direction.toUpperCase() as "ASC" | "DESC",
				...initialFilters,
			};

			loadTransactionsPaginated(filters);
		}
	}, [
		userId,
		pageSize,
		initialFilters,
		loadTransactionsPaginated,
		searchQuery,
		sortConfig.field,
		sortConfig.direction,
	]);

	// Handle search with debouncing
	useEffect(() => {
		const timer = setTimeout(() => {
			if (localSearchQuery !== searchQuery) {
				setSearchQuery(localSearchQuery);
				if (userId) {
					applyFilters({ search: localSearchQuery || undefined });
				}
			}
		}, 300);

		return () => clearTimeout(timer);
	}, [localSearchQuery, searchQuery, setSearchQuery, applyFilters, userId]);

	// Handle sorting
	const handleSort = (field: string) => {
		const newDirection =
			sortConfig.field === field && sortConfig.direction === "desc"
				? "asc"
				: "desc";

		setSortConfig(field, newDirection);

		if (userId) {
			applyFilters({
				sort_by: field,
				sort_direction: newDirection.toUpperCase() as "ASC" | "DESC",
			});
		}
	};

	// Handle pagination
	const handlePageChange = (newPage: number) => {
		if (userId && currentFilters) {
			const newOffset = (newPage - 1) * pageSize;
			applyFilters({ offset: newOffset });
		}
	};

	// Handle bulk selection
	const handleSelectAll = () => {
		if (selectedTransactionIds.length === transactions.length) {
			clearTransactionSelection();
		} else {
			selectAllTransactions();
		}
	};

	// Memoized transaction status badge
	const getStatusBadge = useMemo(
		() => (status: string) => {
			const variants = {
				completed: "default",
				pending: "secondary",
				cancelled: "destructive",
			} as const;

			return (
				<Badge variant={variants[status as keyof typeof variants] || "outline"}>
					{status.charAt(0).toUpperCase() + status.slice(1)}
				</Badge>
			);
		},
		[],
	);

	// Memoized transaction type badge
	const getTypeBadge = useMemo(
		() => (type: string) => {
			const variants = {
				income: "default",
				expense: "destructive",
				transfer: "secondary",
			} as const;

			return (
				<Badge variant={variants[type as keyof typeof variants] || "outline"}>
					{type.charAt(0).toUpperCase() + type.slice(1)}
				</Badge>
			);
		},
		[],
	);

	if (error) {
		return (
			<Card>
				<CardContent className="p-6">
					<div className="text-center text-red-600">
						Error loading transactions: {error.message}
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card>
			<CardHeader>
				<div className="flex items-center justify-between">
					<CardTitle>Transactions</CardTitle>
					<div className="flex items-center gap-2">
						{/* Search */}
						<div className="relative">
							<Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
							<Input
								placeholder="Search transactions..."
								value={localSearchQuery}
								onChange={(e) => setLocalSearchQuery(e.target.value)}
								className="pl-8 w-64"
							/>
						</div>

						{/* Bulk Actions */}
						{showBulkActions && selectedTransactionIds.length > 0 && (
							<div className="flex items-center gap-2">
								<span className="text-sm text-muted-foreground">
									{selectedTransactionIds.length} selected
								</span>
								<Button variant="outline" size="sm">
									<Download className="h-4 w-4 mr-1" />
									Export
								</Button>
								<Button variant="outline" size="sm">
									<Edit className="h-4 w-4 mr-1" />
									Edit
								</Button>
								<Button variant="destructive" size="sm">
									<Trash2 className="h-4 w-4 mr-1" />
									Delete
								</Button>
							</div>
						)}
					</div>
				</div>
			</CardHeader>
			<CardContent>
				<div className="rounded-md border">
					<Table>
						<TableHeader>
							<TableRow>
								{showBulkActions && (
									<TableHead className="w-12">
										<Checkbox
											checked={
												transactions.length > 0 &&
												selectedTransactionIds.length === transactions.length
											}
											onCheckedChange={handleSelectAll}
											aria-label="Select all transactions"
										/>
									</TableHead>
								)}
								<TableHead
									className="cursor-pointer select-none"
									onClick={() => handleSort("transaction_date")}
								>
									<div className="flex items-center">
										Date
										{sortConfig.field === "transaction_date" &&
											(sortConfig.direction === "desc" ? (
												<ChevronDown className="ml-1 h-4 w-4" />
											) : (
												<ChevronUp className="ml-1 h-4 w-4" />
											))}
									</div>
								</TableHead>
								<TableHead>Description</TableHead>
								<TableHead>Category</TableHead>
								<TableHead
									className="cursor-pointer select-none text-right"
									onClick={() => handleSort("amount")}
								>
									<div className="flex items-center justify-end">
										Amount
										{sortConfig.field === "amount" &&
											(sortConfig.direction === "desc" ? (
												<ChevronDown className="ml-1 h-4 w-4" />
											) : (
												<ChevronUp className="ml-1 h-4 w-4" />
											))}
									</div>
								</TableHead>
								<TableHead>Type</TableHead>
								<TableHead>Status</TableHead>
								<TableHead className="w-12"></TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{loadingStates.transactions ? (
								// Loading skeleton
								Array.from(
									{ length: 5 },
									(_, index) => `loading-skeleton-${index}-${Date.now()}`,
								).map((key) => (
									<TableRow key={key}>
										{showBulkActions && (
											<TableCell>
												<div className="h-4 w-4 bg-muted rounded animate-pulse" />
											</TableCell>
										)}
										<TableCell>
											<div className="h-4 w-20 bg-muted rounded animate-pulse" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-32 bg-muted rounded animate-pulse" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-24 bg-muted rounded animate-pulse" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-16 bg-muted rounded animate-pulse ml-auto" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-12 bg-muted rounded animate-pulse" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-16 bg-muted rounded animate-pulse" />
										</TableCell>
										<TableCell>
											<div className="h-4 w-4 bg-muted rounded animate-pulse" />
										</TableCell>
									</TableRow>
								))
							) : transactions.length === 0 ? (
								<TableRow>
									<TableCell
										colSpan={showBulkActions ? 8 : 7}
										className="text-center py-8 text-muted-foreground"
									>
										No transactions found
									</TableCell>
								</TableRow>
							) : (
								transactions.map((transaction) => (
									<TableRow key={transaction.id}>
										{showBulkActions && (
											<TableCell>
												<Checkbox
													checked={selectedTransactionIds.includes(
														transaction.id,
													)}
													onCheckedChange={() =>
														toggleTransactionSelection(transaction.id)
													}
													aria-label={`Select transaction ${transaction.description}`}
												/>
											</TableCell>
										)}
										<TableCell>
											{formatDate(transaction.transaction_date)}
										</TableCell>
										<TableCell className="font-medium">
											{transaction.description}
											{transaction.payee && (
												<div className="text-sm text-muted-foreground">
													{transaction.payee}
												</div>
											)}
										</TableCell>
										<TableCell>
											{transaction.category_id ? (
												<Badge variant="outline">Category</Badge>
											) : (
												<span className="text-muted-foreground">
													Uncategorized
												</span>
											)}
										</TableCell>
										<TableCell className="text-right font-mono">
											<span
												className={
													transaction.transaction_type === "income"
														? "text-green-600"
														: transaction.transaction_type === "expense"
															? "text-red-600"
															: "text-blue-600"
												}
											>
												{transaction.transaction_type === "income"
													? "+"
													: transaction.transaction_type === "expense"
														? "-"
														: ""}
												{formatCurrency(transaction.amount)}
											</span>
										</TableCell>
										<TableCell>
											{getTypeBadge(transaction.transaction_type)}
										</TableCell>
										<TableCell>{getStatusBadge(transaction.status)}</TableCell>
										<TableCell>
											<DropdownMenu>
												<DropdownMenuTrigger asChild>
													<Button variant="ghost" size="sm">
														<MoreHorizontal className="h-4 w-4" />
													</Button>
												</DropdownMenuTrigger>
												<DropdownMenuContent align="end">
													<DropdownMenuItem
														onClick={() => onViewTransaction?.(transaction)}
													>
														<Eye className="h-4 w-4 mr-2" />
														View
													</DropdownMenuItem>
													<DropdownMenuItem
														onClick={() => onEditTransaction?.(transaction)}
													>
														<Edit className="h-4 w-4 mr-2" />
														Edit
													</DropdownMenuItem>
													<DropdownMenuSeparator />
													<DropdownMenuItem className="text-destructive">
														<Trash2 className="h-4 w-4 mr-2" />
														Delete
													</DropdownMenuItem>
												</DropdownMenuContent>
											</DropdownMenu>
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</div>

				{/* Pagination */}
				{showPagination && pagination && pagination.total_pages > 1 && (
					<div className="flex items-center justify-between mt-4">
						<div className="text-sm text-muted-foreground">
							Showing {(pagination.page - 1) * pagination.per_page + 1} to{" "}
							{Math.min(
								pagination.page * pagination.per_page,
								pagination.total,
							)}{" "}
							of {pagination.total} transactions
						</div>
						<div className="flex items-center gap-2">
							<Button
								variant="outline"
								size="sm"
								onClick={() => handlePageChange(pagination.page - 1)}
								disabled={pagination.page <= 1}
							>
								Previous
							</Button>
							<span className="text-sm">
								Page {pagination.page} of {pagination.total_pages}
							</span>
							<Button
								variant="outline"
								size="sm"
								onClick={() => handlePageChange(pagination.page + 1)}
								disabled={pagination.page >= pagination.total_pages}
							>
								Next
							</Button>
						</div>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
