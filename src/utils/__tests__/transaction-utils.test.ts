/**
 * Tests for transaction utility functions
 */

import { describe, expect, it } from "vitest";
import {
	prepareTransactionData,
	type TransactionFormData,
} from "../transaction-utils";

describe("prepareTransactionData", () => {
	const mockFormData: TransactionFormData = {
		account_id: "account-123",
		category_id: "category-456",
		amount: 100.5,
		description: "Test transaction",
		notes: "Test notes",
		transaction_date: new Date("2024-01-15T10:30:00Z"),
		transaction_type: "expense",
		payee: "Test Payee",
		reference_number: "REF-123",
	};

	it("should prepare transaction data with all fields", () => {
		const result = prepareTransactionData(mockFormData);

		expect(result).toEqual({
			category_id: "category-456",
			amount: 100.5,
			description: "Test transaction",
			notes: "Test notes",
			transaction_date: "2024-01-15T10:30:00.000Z",
			transaction_type: "expense",
			payee: "Test Payee",
			reference_number: "REF-123",
		});
	});

	it("should convert category_id 'no-category' to undefined", () => {
		const formData = {
			...mockFormData,
			category_id: "no-category",
		};

		const result = prepareTransactionData(formData);

		expect(result.category_id).toBeUndefined();
	});

	it("should convert undefined category_id to undefined", () => {
		const formData = {
			...mockFormData,
			category_id: undefined,
		};

		const result = prepareTransactionData(formData);

		expect(result.category_id).toBeUndefined();
	});

	it("should convert empty string notes to undefined", () => {
		const formData = {
			...mockFormData,
			notes: "",
		};

		const result = prepareTransactionData(formData);

		expect(result.notes).toBeUndefined();
	});

	it("should convert undefined notes to undefined", () => {
		const formData = {
			...mockFormData,
			notes: undefined,
		};

		const result = prepareTransactionData(formData);

		expect(result.notes).toBeUndefined();
	});

	it("should convert empty string payee to undefined", () => {
		const formData = {
			...mockFormData,
			payee: "",
		};

		const result = prepareTransactionData(formData);

		expect(result.payee).toBeUndefined();
	});

	it("should convert undefined payee to undefined", () => {
		const formData = {
			...mockFormData,
			payee: undefined,
		};

		const result = prepareTransactionData(formData);

		expect(result.payee).toBeUndefined();
	});

	it("should convert empty string reference_number to undefined", () => {
		const formData = {
			...mockFormData,
			reference_number: "",
		};

		const result = prepareTransactionData(formData);

		expect(result.reference_number).toBeUndefined();
	});

	it("should convert undefined reference_number to undefined", () => {
		const formData = {
			...mockFormData,
			reference_number: undefined,
		};

		const result = prepareTransactionData(formData);

		expect(result.reference_number).toBeUndefined();
	});

	it("should convert Date object to ISO string", () => {
		const testDate = new Date("2024-03-20T14:45:30.123Z");
		const formData = {
			...mockFormData,
			transaction_date: testDate,
		};

		const result = prepareTransactionData(formData);

		expect(result.transaction_date).toBe("2024-03-20T14:45:30.123Z");
	});

	it("should handle income transaction type", () => {
		const formData = {
			...mockFormData,
			transaction_type: "income" as const,
		};

		const result = prepareTransactionData(formData);

		expect(result.transaction_type).toBe("income");
	});

	it("should handle transfer transaction type", () => {
		const formData = {
			...mockFormData,
			transaction_type: "transfer" as const,
		};

		const result = prepareTransactionData(formData);

		expect(result.transaction_type).toBe("transfer");
	});

	it("should handle minimal form data with only required fields", () => {
		const minimalFormData: TransactionFormData = {
			account_id: "account-123",
			amount: 50.25,
			description: "Minimal transaction",
			transaction_date: new Date("2024-01-01T00:00:00Z"),
			transaction_type: "expense",
		};

		const result = prepareTransactionData(minimalFormData);

		expect(result).toEqual({
			category_id: undefined,
			amount: 50.25,
			description: "Minimal transaction",
			notes: undefined,
			transaction_date: "2024-01-01T00:00:00.000Z",
			transaction_type: "expense",
			payee: undefined,
			reference_number: undefined,
		});
	});
});
