/**
 * Transactions Page
 * Main page for managing transactions with comprehensive features
 */

import { BarChart3, Download, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { TransactionFilters } from "@/components/transactions/TransactionFilters";
import { TransactionForm } from "@/components/transactions/TransactionForm";
import { TransactionList } from "@/components/transactions/TransactionList";
import { TransactionStats } from "@/components/transactions/TransactionStats";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useUserId } from "@/stores/auth-store";
import { useTransactionsStore } from "@/stores/transactions-store";
import type { TransactionFilters as TFilters, Transaction } from "@/types/api";

export function TransactionsPage() {
	const userId = useUserId();
	const {
		selectedTransactionIds,
		clearTransactionSelection,
		bulkOperations,
		loadingStates,
	} = useTransactionsStore();

	const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
	const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
	const [editingTransaction, setEditingTransaction] =
		useState<Transaction | null>(null);
	const [activeTab, setActiveTab] = useState("list");
	const [filters, setFilters] = useState<Partial<TFilters>>({});

	// Initialize with user ID when available
	useEffect(() => {
		if (userId) {
			setFilters((prev) => ({ ...prev, user_id: userId }));
		}
	}, [userId]);

	// Handle transaction form success
	const handleTransactionSuccess = (_transaction: Transaction) => {
		setIsAddDialogOpen(false);
		setIsEditDialogOpen(false);
		setEditingTransaction(null);
		clearTransactionSelection();
	};

	// Handle edit transaction
	const handleEditTransaction = (transaction: Transaction) => {
		setEditingTransaction(transaction);
		setIsEditDialogOpen(true);
	};

	// Handle view transaction (could open a detail view)
	const handleViewTransaction = (transaction: Transaction) => {
		// For now, just edit the transaction
		handleEditTransaction(transaction);
	};

	// Handle filters change
	const handleFiltersChange = (newFilters: Partial<TFilters>) => {
		if (userId) {
			setFilters((prev) => ({
				...prev,
				...newFilters,
				user_id: userId, // Always include user ID
			}));
		}
	};

	// Handle bulk export
	const handleBulkExport = async (format: "csv" | "json") => {
		if (selectedTransactionIds.length === 0 || !userId) return;

		try {
			const result = await bulkOperations({
				user_id: userId,
				transaction_ids: selectedTransactionIds,
				action: { type: "export", format },
			});

			if (result) {
				// Create and download file
				const blob = new Blob([result], {
					type: format === "csv" ? "text/csv" : "application/json",
				});
				const url = URL.createObjectURL(blob);
				const a = document.createElement("a");
				a.href = url;
				a.download = `transactions.${format}`;
				document.body.appendChild(a);
				a.click();
				document.body.removeChild(a);
				URL.revokeObjectURL(url);
			}
		} catch (error) {
			console.error("Export failed:", error);
		}
	};

	// Handle bulk delete
	const handleBulkDelete = async () => {
		if (selectedTransactionIds.length === 0 || !userId) return;

		if (
			confirm(
				`Are you sure you want to delete ${selectedTransactionIds.length} transactions?`,
			)
		) {
			try {
				await bulkOperations({
					user_id: userId,
					transaction_ids: selectedTransactionIds,
					action: { type: "delete" },
				});
			} catch (error) {
				console.error("Delete failed:", error);
			}
		}
	};

	return (
		<div className="container mx-auto p-6 space-y-6">
			{/* Page Header */}
			<div className="flex items-center justify-between">
				<div>
					<h1 className="text-3xl font-bold tracking-tight">Transactions</h1>
					<p className="text-muted-foreground">
						Manage your income, expenses, and transfers
					</p>
				</div>
				<div className="flex items-center gap-2">
					{/* Bulk Actions */}
					{selectedTransactionIds.length > 0 && (
						<>
							<Button
								variant="outline"
								size="sm"
								onClick={() => handleBulkExport("csv")}
								disabled={loadingStates.bulk}
							>
								<Download className="h-4 w-4 mr-1" />
								Export CSV
							</Button>
							<Button
								variant="outline"
								size="sm"
								onClick={() => handleBulkExport("json")}
								disabled={loadingStates.bulk}
							>
								<Download className="h-4 w-4 mr-1" />
								Export JSON
							</Button>
							<Button
								variant="destructive"
								size="sm"
								onClick={handleBulkDelete}
								disabled={loadingStates.bulk}
							>
								Delete ({selectedTransactionIds.length})
							</Button>
						</>
					)}

					{/* Add Transaction */}
					<Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
						<DialogTrigger asChild>
							<Button>
								<Plus className="h-4 w-4 mr-2" />
								Add Transaction
							</Button>
						</DialogTrigger>
						<DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
							<DialogHeader>
								<DialogTitle>Add New Transaction</DialogTitle>
							</DialogHeader>
							<TransactionForm
								onSuccess={handleTransactionSuccess}
								onCancel={() => setIsAddDialogOpen(false)}
								isModal={true}
							/>
						</DialogContent>
					</Dialog>

					{/* Edit Transaction Dialog */}
					<Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
						<DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
							<DialogHeader>
								<DialogTitle>Edit Transaction</DialogTitle>
							</DialogHeader>
							{editingTransaction && (
								<TransactionForm
									transaction={editingTransaction}
									onSuccess={handleTransactionSuccess}
									onCancel={() => setIsEditDialogOpen(false)}
									isModal={true}
								/>
							)}
						</DialogContent>
					</Dialog>
				</div>
			</div>

			{/* Main Content */}
			<Tabs
				value={activeTab}
				onValueChange={setActiveTab}
				className="space-y-6"
			>
				<TabsList className="grid w-full grid-cols-2">
					<TabsTrigger value="list" className="flex items-center gap-2">
						<BarChart3 className="h-4 w-4" />
						Transactions
					</TabsTrigger>
					<TabsTrigger value="stats" className="flex items-center gap-2">
						<BarChart3 className="h-4 w-4" />
						Statistics
					</TabsTrigger>
				</TabsList>

				<TabsContent value="list" className="space-y-6">
					{/* Filters */}
					<TransactionFilters
						filters={filters}
						onFiltersChange={handleFiltersChange}
						defaultExpanded={false}
					/>

					{/* Transaction List */}
					<TransactionList
						initialFilters={filters}
						showBulkActions={true}
						showPagination={true}
						pageSize={50}
						onEditTransaction={handleEditTransaction}
						onViewTransaction={handleViewTransaction}
					/>
				</TabsContent>

				<TabsContent value="stats" className="space-y-6">
					{/* Statistics */}
					<TransactionStats filters={filters} showDetails={true} />
				</TabsContent>
			</Tabs>
		</div>
	);
}
