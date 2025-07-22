/**
 * Transaction utility functions
 * Helper functions for transaction data preparation and transformation
 */

import type { TransactionType } from "@/types/api";

/**
 * Form data structure from the transaction form
 */
export interface TransactionFormData {
	account_id: string;
	category_id?: string;
	amount: number;
	description: string;
	notes?: string;
	transaction_date: Date;
	transaction_type: TransactionType;
	payee?: string;
	reference_number?: string;
}

/**
 * Base transaction data structure with common fields
 * Used as foundation for both create and update operations
 */
export interface BaseTransactionData {
	category_id?: string;
	amount: number;
	description: string;
	notes?: string;
	transaction_date: string;
	transaction_type: TransactionType;
	payee?: string;
	reference_number?: string;
}

/**
 * Prepares transaction data from form values for API operations
 * Handles common data transformations including:
 * - Converting category_id "no-category" to undefined
 * - Converting Date object to ISO string
 * - Converting empty optional strings to undefined
 *
 * @param data - Form data from the transaction form
 * @returns Prepared transaction data ready for API calls
 */
export function prepareTransactionData(
	data: TransactionFormData,
): BaseTransactionData {
	return {
		category_id:
			data.category_id && data.category_id !== "no-category"
				? data.category_id
				: undefined,
		amount: data.amount,
		description: data.description,
		notes: data.notes?.trim() === "" ? undefined : data.notes,
		transaction_date: data.transaction_date.toISOString(),
		transaction_type: data.transaction_type,
		payee: data.payee?.trim() === "" ? undefined : data.payee,
		reference_number:
			data.reference_number?.trim() === "" ? undefined : data.reference_number,
	};
}
