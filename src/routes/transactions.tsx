/**
 * Transactions Route
 * Main transactions route with layout and page components
 */

import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { DashboardLayout } from "@/components/layout/DashboardLayout";
import { TransactionsPage } from "@/pages/TransactionsPage";

// Search params validation schema
const transactionsSearchSchema = z.object({
	// Pagination
	page: z.number().min(1).optional().default(1),
	limit: z.number().min(1).max(1000).optional().default(50),

	// Filters
	search: z.string().optional(),
	account_id: z.string().uuid().optional(),
	category_id: z.string().uuid().optional(),
	transaction_type: z.enum(["income", "expense", "transfer"]).optional(),
	status: z.enum(["completed", "pending", "cancelled"]).optional(),

	// Date filters
	start_date: z.string().datetime().optional(),
	end_date: z.string().datetime().optional(),
	date_preset: z
		.enum(["today", "yesterday", "week", "month", "quarter", "year"])
		.optional(),

	// Amount filters
	min_amount: z.number().min(0).optional(),
	max_amount: z.number().min(0).optional(),

	// Sorting
	sort_by: z
		.enum([
			"amount",
			"description",
			"transaction_date",
			"created_at",
			"updated_at",
		])
		.optional()
		.default("transaction_date"),
	sort_direction: z.enum(["asc", "desc"]).optional().default("desc"),

	// View options
	view: z.enum(["list", "stats"]).optional().default("list"),
});

export const Route = createFileRoute("/transactions")({
	component: Transactions,
	validateSearch: transactionsSearchSchema,
	// Preload transaction data based on search params
	beforeLoad: ({ search }) => {
		// You can add authentication checks here
		// For now, we'll just pass through the search params
		return {
			searchParams: search,
		};
	},
});

function Transactions() {
	return (
		<DashboardLayout>
			<TransactionsPage />
		</DashboardLayout>
	);
}
