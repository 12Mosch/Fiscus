/**
 * Transaction Filters Component
 * Advanced filtering UI for transactions with date ranges, amounts, categories, etc.
 */

import { zodResolver } from "@hookform/resolvers/zod";
import { format } from "date-fns";
import { CalendarIcon, Filter, Search, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from "@/components/ui/collapsible";
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
import { cn } from "@/lib/utils";
import { useAccountsStore } from "@/stores/accounts-store";
import { useUserId } from "@/stores/auth-store";
import { useCategoriesStore } from "@/stores/categories-store";
import type {
	TransactionFilters as TransactionFiltersType,
	TransactionStatus,
	TransactionType,
} from "@/types/api";
import { getDateRange } from "@/utils/date";

// Filter form validation schema
const filterFormSchema = z.object({
	search: z.string().optional(),
	account_id: z.string().optional(),
	category_id: z.string().optional(),
	transaction_type: z
		.enum(["income", "expense", "transfer", "all-types"])
		.optional(),
	status: z
		.enum(["completed", "pending", "cancelled", "all-statuses"])
		.optional(),
	start_date: z.date().optional(),
	end_date: z.date().optional(),
	min_amount: z.number().min(0).optional(),
	max_amount: z.number().min(0).optional(),
	date_preset: z.string().optional(),
});

type FilterFormData = z.infer<typeof filterFormSchema>;

interface TransactionFiltersProps {
	/** Current filters */
	filters: Partial<TransactionFiltersType>;
	/** Callback when filters change */
	onFiltersChange: (filters: Partial<TransactionFiltersType>) => void;
	/** Whether to show the filters expanded by default */
	defaultExpanded?: boolean;
}

export function TransactionFilters({
	filters,
	onFiltersChange,
	defaultExpanded = false,
}: TransactionFiltersProps) {
	const userId = useUserId();
	const { accounts, loadAccounts } = useAccountsStore();
	const { categories, loadCategories } = useCategoriesStore();

	const [isExpanded, setIsExpanded] = useState(defaultExpanded);
	const [activeFiltersCount, setActiveFiltersCount] = useState(0);

	const form = useForm<FilterFormData>({
		resolver: zodResolver(filterFormSchema),
		defaultValues: {
			search: filters.search || "",
			account_id: filters.account_id || "all-accounts",
			category_id: filters.category_id || "all-categories",
			transaction_type:
				(filters.transaction_type as TransactionType) || "all-types",
			status: (filters.status as TransactionStatus) || "all-statuses",
			start_date: filters.start_date ? new Date(filters.start_date) : undefined,
			end_date: filters.end_date ? new Date(filters.end_date) : undefined,
			min_amount: filters.min_amount,
			max_amount: filters.max_amount,
			date_preset: "custom-range",
		},
	});

	// Load accounts and categories on mount
	useEffect(() => {
		if (userId) {
			loadAccounts({ user_id: userId });
			loadCategories({ user_id: userId });
		}
	}, [userId, loadAccounts, loadCategories]);

	// Count active filters
	useEffect(() => {
		const values = form.watch();
		let count = 0;

		if (values.search) count++;
		if (values.account_id && values.account_id !== "all-accounts") count++;
		if (values.category_id && values.category_id !== "all-categories") count++;
		if (values.transaction_type && values.transaction_type !== "all-types")
			count++;
		if (values.status && values.status !== "all-statuses") count++;
		if (values.start_date || values.end_date) count++;
		if (values.min_amount !== undefined || values.max_amount !== undefined)
			count++;

		setActiveFiltersCount(count);
	}, [form.watch]);

	// Apply filters
	const applyFilters = (data: FilterFormData) => {
		const newFilters: Partial<TransactionFiltersType> = {};

		if (data.search) newFilters.search = data.search;
		if (data.account_id && data.account_id !== "all-accounts")
			newFilters.account_id = data.account_id;
		if (data.category_id && data.category_id !== "all-categories")
			newFilters.category_id = data.category_id;
		if (data.transaction_type && data.transaction_type !== "all-types")
			newFilters.transaction_type = data.transaction_type;
		if (data.status && data.status !== "all-statuses")
			newFilters.status = data.status;
		if (data.start_date) newFilters.start_date = data.start_date.toISOString();
		if (data.end_date) newFilters.end_date = data.end_date.toISOString();
		if (data.min_amount !== undefined) newFilters.min_amount = data.min_amount;
		if (data.max_amount !== undefined) newFilters.max_amount = data.max_amount;

		onFiltersChange(newFilters);
	};

	// Clear all filters
	const clearFilters = () => {
		form.reset({
			search: "",
			account_id: "all-accounts",
			category_id: "all-categories",
			transaction_type: "all-types",
			status: "all-statuses",
			start_date: undefined,
			end_date: undefined,
			min_amount: undefined,
			max_amount: undefined,
			date_preset: "custom-range",
		});
		onFiltersChange({});
	};

	// Handle date preset selection
	const handleDatePreset = (preset: string) => {
		if (!preset || preset === "custom-range") return;

		const { start, end } = getDateRange(
			preset as "today" | "yesterday" | "week" | "month" | "quarter" | "year",
		);
		form.setValue("start_date", start);
		form.setValue("end_date", end);
		form.setValue("date_preset", preset);
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

	const datePresets = [
		{ value: "today", label: "Today" },
		{ value: "yesterday", label: "Yesterday" },
		{ value: "week", label: "This Week" },
		{ value: "month", label: "This Month" },
		{ value: "quarter", label: "This Quarter" },
		{ value: "year", label: "This Year" },
	];

	return (
		<Card>
			<Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
				<CardHeader className="pb-3">
					<div className="flex items-center justify-between">
						<div className="flex items-center gap-2">
							<CollapsibleTrigger asChild>
								<Button variant="ghost" size="sm">
									<Filter className="h-4 w-4 mr-2" />
									Filters
									{activeFiltersCount > 0 && (
										<Badge variant="secondary" className="ml-2">
											{activeFiltersCount}
										</Badge>
									)}
								</Button>
							</CollapsibleTrigger>
						</div>
						{activeFiltersCount > 0 && (
							<Button variant="outline" size="sm" onClick={clearFilters}>
								<X className="h-4 w-4 mr-1" />
								Clear All
							</Button>
						)}
					</div>
				</CardHeader>

				<CollapsibleContent>
					<CardContent>
						<Form {...form}>
							<form
								onSubmit={form.handleSubmit(applyFilters)}
								className="space-y-6"
							>
								{/* Search */}
								<FormField
									control={form.control}
									name="search"
									render={({ field }) => (
										<FormItem>
											<FormLabel>Search</FormLabel>
											<FormControl>
												<div className="relative">
													<Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
													<Input
														placeholder="Search descriptions, payees, notes..."
														className="pl-8"
														{...field}
													/>
												</div>
											</FormControl>
											<FormMessage />
										</FormItem>
									)}
								/>

								<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
									{/* Account Filter */}
									<FormField
										control={form.control}
										name="account_id"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Account</FormLabel>
												<Select
													onValueChange={field.onChange}
													value={field.value}
												>
													<FormControl>
														<SelectTrigger>
															<SelectValue placeholder="All accounts" />
														</SelectTrigger>
													</FormControl>
													<SelectContent>
														<SelectItem value="all-accounts">
															All accounts
														</SelectItem>
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

									{/* Category Filter */}
									<FormField
										control={form.control}
										name="category_id"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Category</FormLabel>
												<Select
													onValueChange={field.onChange}
													value={field.value}
												>
													<FormControl>
														<SelectTrigger>
															<SelectValue placeholder="All categories" />
														</SelectTrigger>
													</FormControl>
													<SelectContent>
														<SelectItem value="all-categories">
															All categories
														</SelectItem>
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

									{/* Transaction Type Filter */}
									<FormField
										control={form.control}
										name="transaction_type"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Type</FormLabel>
												<Select
													onValueChange={field.onChange}
													value={field.value}
												>
													<FormControl>
														<SelectTrigger>
															<SelectValue placeholder="All types" />
														</SelectTrigger>
													</FormControl>
													<SelectContent>
														<SelectItem value="all-types">All types</SelectItem>
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

									{/* Status Filter */}
									<FormField
										control={form.control}
										name="status"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Status</FormLabel>
												<Select
													onValueChange={field.onChange}
													value={field.value}
												>
													<FormControl>
														<SelectTrigger>
															<SelectValue placeholder="All statuses" />
														</SelectTrigger>
													</FormControl>
													<SelectContent>
														<SelectItem value="all-statuses">
															All statuses
														</SelectItem>
														{transactionStatuses.map((status) => (
															<SelectItem
																key={status.value}
																value={status.value}
															>
																{status.label}
															</SelectItem>
														))}
													</SelectContent>
												</Select>
												<FormMessage />
											</FormItem>
										)}
									/>

									{/* Date Preset */}
									<FormField
										control={form.control}
										name="date_preset"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Date Range</FormLabel>
												<Select
													onValueChange={(value: string) => {
														field.onChange(value);
														handleDatePreset(value);
													}}
													value={field.value}
												>
													<FormControl>
														<SelectTrigger>
															<SelectValue placeholder="Select date range" />
														</SelectTrigger>
													</FormControl>
													<SelectContent>
														<SelectItem value="custom-range">
															Custom range
														</SelectItem>
														{datePresets.map((preset) => (
															<SelectItem
																key={preset.value}
																value={preset.value}
															>
																{preset.label}
															</SelectItem>
														))}
													</SelectContent>
												</Select>
												<FormMessage />
											</FormItem>
										)}
									/>
								</div>

								{/* Custom Date Range */}
								<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
									<FormField
										control={form.control}
										name="start_date"
										render={({ field }) => (
											<FormItem className="flex flex-col">
												<FormLabel>Start Date</FormLabel>
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
																	<span>Pick start date</span>
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
																date > new Date() ||
																date < new Date("1900-01-01")
															}
															initialFocus
														/>
													</PopoverContent>
												</Popover>
												<FormMessage />
											</FormItem>
										)}
									/>

									<FormField
										control={form.control}
										name="end_date"
										render={({ field }) => (
											<FormItem className="flex flex-col">
												<FormLabel>End Date</FormLabel>
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
																	<span>Pick end date</span>
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
																date > new Date() ||
																date < new Date("1900-01-01")
															}
															initialFocus
														/>
													</PopoverContent>
												</Popover>
												<FormMessage />
											</FormItem>
										)}
									/>
								</div>

								{/* Amount Range */}
								<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
									<FormField
										control={form.control}
										name="min_amount"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Minimum Amount</FormLabel>
												<FormControl>
													<Input
														type="number"
														step="0.01"
														min="0"
														placeholder="0.00"
														{...field}
														onChange={(e) =>
															field.onChange(
																e.target.value === ""
																	? undefined
																	: Number.isNaN(Number.parseFloat(e.target.value))
																		? undefined
																		: Number.parseFloat(e.target.value),
															)
														}
													/>
												</FormControl>
												<FormMessage />
											</FormItem>
										)}
									/>

									<FormField
										control={form.control}
										name="max_amount"
										render={({ field }) => (
											<FormItem>
												<FormLabel>Maximum Amount</FormLabel>
												<FormControl>
													<Input
														type="number"
														step="0.01"
														min="0"
														placeholder="0.00"
														{...field}
														onChange={(e) =>
															field.onChange(
																e.target.value === ""
																	? undefined
																	: Number.isNaN(Number.parseFloat(e.target.value))
																		? undefined
																		: Number.parseFloat(e.target.value),
															)
														}
													/>
												</FormControl>
												<FormMessage />
											</FormItem>
										)}
									/>
								</div>

								{/* Form Actions */}
								<div className="flex justify-end gap-3">
									<Button
										type="button"
										variant="outline"
										onClick={clearFilters}
									>
										Clear
									</Button>
									<Button type="submit">Apply Filters</Button>
								</div>
							</form>
						</Form>
					</CardContent>
				</CollapsibleContent>
			</Collapsible>
		</Card>
	);
}
