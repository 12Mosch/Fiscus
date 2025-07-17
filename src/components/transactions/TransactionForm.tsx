/**
 * Transaction Form Component
 * Form for creating and editing transactions with validation
 */

import { zodResolver } from "@hookform/resolvers/zod";
import { format } from "date-fns";
import { CalendarIcon, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { type FieldValues, type UseFormReturn, useForm } from "react-hook-form";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
import { useAccountsStore } from "@/stores/accounts-store";
import { useUserId } from "@/stores/auth-store";
import { useCategoriesStore } from "@/stores/categories-store";
import { useTransactionsStore } from "@/stores/transactions-store";
import type {
	Transaction,
	TransactionStatus,
	TransactionType,
} from "@/types/api";

// Form validation schema
const transactionFormSchema = z.object({
	account_id: z.string().min(1, "Account is required"),
	category_id: z.string().optional(),
	amount: z.number().positive("Amount must be positive"),
	description: z
		.string()
		.min(1, "Description is required")
		.max(255, "Description too long"),
	notes: z.string().max(500, "Notes too long").optional(),
	transaction_date: z.date(),
	transaction_type: z.enum(["income", "expense", "transfer"]),
	status: z.enum(["completed", "pending", "cancelled"]),
	payee: z.string().max(255, "Payee name too long").optional(),
	reference_number: z.string().max(100, "Reference number too long").optional(),
});

type TransactionFormData = z.infer<typeof transactionFormSchema>;

interface TransactionFormProps {
	/** Transaction to edit (if editing) */
	transaction?: Transaction;
	/** Callback when form is submitted successfully */
	onSuccess?: (transaction: Transaction) => void;
	/** Callback when form is cancelled */
	onCancel?: () => void;
	/** Whether to show as a modal */
	isModal?: boolean;
}

export function TransactionForm({
	transaction,
	onSuccess,
	onCancel,
	isModal = false,
}: TransactionFormProps) {
	const userId = useUserId();
	const { createTransaction, updateTransaction, loading } =
		useTransactionsStore();
	const { accounts, loadAccounts } = useAccountsStore();
	const { categories, loadCategories } = useCategoriesStore();

	const [isSubmitting, setIsSubmitting] = useState(false);

	const form = useForm<TransactionFormData>({
		resolver: zodResolver(transactionFormSchema),
		defaultValues: {
			account_id: transaction?.account_id || "",
			category_id: transaction?.category_id || "no-category",
			amount: transaction?.amount || 0,
			description: transaction?.description || "",
			notes: transaction?.notes || "",
			transaction_date: transaction
				? new Date(transaction.transaction_date)
				: new Date(),
			transaction_type:
				(transaction?.transaction_type as TransactionType) || "expense",
			status: (transaction?.status as TransactionStatus) || "completed",
			payee: transaction?.payee || "",
			reference_number: transaction?.reference_number || "",
		},
	});

	// Load accounts and categories on mount
	useEffect(() => {
		if (userId) {
			loadAccounts({ user_id: userId });
			loadCategories({ user_id: userId });
		}
	}, [userId, loadAccounts, loadCategories]);

	const onSubmit = async (data: TransactionFormData) => {
		if (!userId) return;

		setIsSubmitting(true);

		try {
			let result: Transaction | null = null;

			if (transaction) {
				// Update existing transaction
				result = await updateTransaction(transaction.id, userId, {
					category_id:
						data.category_id && data.category_id !== "no-category"
							? data.category_id
							: undefined,
					amount: data.amount,
					description: data.description,
					notes: data.notes || undefined,
					transaction_date: data.transaction_date.toISOString(),
					transaction_type: data.transaction_type,
					payee: data.payee || undefined,
					reference_number: data.reference_number || undefined,
				});
			} else {
				// Create new transaction
				result = await createTransaction({
					user_id: userId,
					account_id: data.account_id,
					category_id:
						data.category_id && data.category_id !== "no-category"
							? data.category_id
							: undefined,
					amount: data.amount,
					description: data.description,
					notes: data.notes || undefined,
					transaction_date: data.transaction_date.toISOString(),
					transaction_type: data.transaction_type,
					payee: data.payee || undefined,
					reference_number: data.reference_number || undefined,
				});
			}

			if (result) {
				onSuccess?.(result);
				if (!transaction) {
					// Reset form for new transactions
					form.reset();
				}
			}
		} catch (error) {
			console.error("Failed to save transaction:", error);
		} finally {
			setIsSubmitting(false);
		}
	};

	const transactionTypes: { value: TransactionType; label: string }[] = [
		{ value: "income", label: "Income" },
		{ value: "expense", label: "Expense" },
		{ value: "transfer", label: "Transfer" },
	];

	const transactionStatuses: { value: TransactionStatus; label: string }[] = [
		{ value: "completed", label: "Completed" },
		{ value: "pending", label: "Pending" },
		{ value: "cancelled", label: "Cancelled" },
	];

	const content = (
		<Form {...(form as unknown as UseFormReturn<FieldValues>)}>
			<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
				<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
					{/* Account Selection */}
					<FormField
						control={form.control}
						name="account_id"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Account</FormLabel>
								<Select
									onValueChange={field.onChange}
									defaultValue={field.value}
								>
									<FormControl>
										<SelectTrigger>
											<SelectValue placeholder="Select an account" />
										</SelectTrigger>
									</FormControl>
									<SelectContent>
										{accounts.map((account) => (
											<SelectItem key={account.id} value={account.id}>
												{account.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
								<FormMessage />
							</FormItem>
						)}
					/>

					{/* Category Selection */}
					<FormField
						control={form.control}
						name="category_id"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Category</FormLabel>
								<Select
									onValueChange={field.onChange}
									defaultValue={field.value}
								>
									<FormControl>
										<SelectTrigger>
											<SelectValue placeholder="Select a category (optional)" />
										</SelectTrigger>
									</FormControl>
									<SelectContent>
										<SelectItem value="no-category">No category</SelectItem>
										{categories.map((category) => (
											<SelectItem key={category.id} value={category.id}>
												{category.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
								<FormMessage />
							</FormItem>
						)}
					/>

					{/* Amount */}
					<FormField
						control={form.control}
						name="amount"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Amount</FormLabel>
								<FormControl>
									<Input
										type="number"
										step="0.01"
										min="0"
										placeholder="0.00"
										{...field}
										onChange={(e) =>
											field.onChange(Number.parseFloat(e.target.value) || 0)
										}
									/>
								</FormControl>
								<FormMessage />
							</FormItem>
						)}
					/>

					{/* Transaction Date */}
					<FormField
						control={form.control}
						name="transaction_date"
						render={({ field }) => (
							<FormItem className="flex flex-col">
								<FormLabel>Transaction Date</FormLabel>
								<Popover>
									<PopoverTrigger asChild>
										<FormControl>
											<Button
												variant="outline"
												className={cn(
													"w-full pl-3 text-left font-normal",
													!field.value && "text-muted-foreground",
												)}
											>
												{field.value ? (
													format(field.value, "PPP")
												) : (
													<span>Pick a date</span>
												)}
												<CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
											</Button>
										</FormControl>
									</PopoverTrigger>
									<PopoverContent className="w-auto p-0" align="start">
										<Calendar
											mode="single"
											selected={field.value}
											onSelect={field.onChange}
											disabled={(date) =>
												date > new Date() || date < new Date("1900-01-01")
											}
											initialFocus
										/>
									</PopoverContent>
								</Popover>
								<FormMessage />
							</FormItem>
						)}
					/>

					{/* Transaction Type */}
					<FormField
						control={form.control}
						name="transaction_type"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Type</FormLabel>
								<Select
									onValueChange={field.onChange}
									defaultValue={field.value}
								>
									<FormControl>
										<SelectTrigger>
											<SelectValue placeholder="Select transaction type" />
										</SelectTrigger>
									</FormControl>
									<SelectContent>
										{transactionTypes.map((type) => (
											<SelectItem key={type.value} value={type.value}>
												{type.label}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
								<FormMessage />
							</FormItem>
						)}
					/>

					{/* Status */}
					<FormField
						control={form.control}
						name="status"
						render={({ field }) => (
							<FormItem>
								<FormLabel>Status</FormLabel>
								<Select
									onValueChange={field.onChange}
									defaultValue={field.value}
								>
									<FormControl>
										<SelectTrigger>
											<SelectValue placeholder="Select status" />
										</SelectTrigger>
									</FormControl>
									<SelectContent>
										{transactionStatuses.map((status) => (
											<SelectItem key={status.value} value={status.value}>
												{status.label}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
								<FormMessage />
							</FormItem>
						)}
					/>
				</div>

				{/* Description */}
				<FormField
					control={form.control}
					name="description"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Description</FormLabel>
							<FormControl>
								<Input placeholder="Transaction description" {...field} />
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				{/* Payee */}
				<FormField
					control={form.control}
					name="payee"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Payee (Optional)</FormLabel>
							<FormControl>
								<Input placeholder="Who was paid or who paid" {...field} />
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				{/* Reference Number */}
				<FormField
					control={form.control}
					name="reference_number"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Reference Number (Optional)</FormLabel>
							<FormControl>
								<Input
									placeholder="Check number, confirmation code, etc."
									{...field}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				{/* Notes */}
				<FormField
					control={form.control}
					name="notes"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Notes (Optional)</FormLabel>
							<FormControl>
								<Textarea
									placeholder="Additional notes about this transaction"
									className="resize-none"
									{...field}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				{/* Form Actions */}
				<div className="flex justify-end gap-3">
					{onCancel && (
						<Button type="button" variant="outline" onClick={onCancel}>
							Cancel
						</Button>
					)}
					<Button type="submit" disabled={isSubmitting || loading}>
						{isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
						{transaction ? "Update Transaction" : "Create Transaction"}
					</Button>
				</div>
			</form>
		</Form>
	);

	if (isModal) {
		return content;
	}

	return (
		<Card>
			<CardHeader>
				<CardTitle>
					{transaction ? "Edit Transaction" : "Add New Transaction"}
				</CardTitle>
			</CardHeader>
			<CardContent>{content}</CardContent>
		</Card>
	);
}
