/**
 * Tests for validation utilities
 */

import { describe, expect, it } from "vitest";
import type {
	CreateAccountRequest,
	CreateTransactionRequest,
	CreateTransferRequest,
	CreateUserRequest,
} from "../../types/api";
import {
	createFormValidator,
	SecurityUtils,
	Validator,
	validateCreateAccountRequest,
	validateCreateTransactionRequest,
	validateCreateTransferRequest,
	validateCreateUserRequest,
} from "../validation";

describe("Validator", () => {
	describe("validateString", () => {
		it("should validate required strings correctly", () => {
			const errors = Validator.validateString("test", "field", 2, 10);
			expect(errors).toHaveLength(0);
		});

		it("should reject empty required strings", () => {
			const errors = Validator.validateString("", "field", 2, 10);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("REQUIRED");
		});

		it("should reject strings that are too short", () => {
			const errors = Validator.validateString("a", "field", 2, 10);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("MIN_LENGTH");
		});

		it("should reject strings that are too long", () => {
			const errors = Validator.validateString("a".repeat(11), "field", 2, 10);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("MAX_LENGTH");
		});

		it("should allow empty optional strings", () => {
			const errors = Validator.validateString("", "field", 2, 10, false);
			expect(errors).toHaveLength(0);
		});
	});

	describe("validateEmail", () => {
		it("should validate correct email formats", () => {
			const validEmails = [
				"test@example.com",
				"user.name@domain.co.uk",
				"user+tag@example.org",
			];

			for (const email of validEmails) {
				const errors = Validator.validateEmail(email);
				expect(errors).toHaveLength(0);
			}
		});

		it("should reject invalid email formats", () => {
			const invalidEmails = [
				"invalid-email",
				"@example.com",
				"user@",
				"user@.com",
				"user..name@example.com",
			];

			for (const email of invalidEmails) {
				const errors = Validator.validateEmail(email);
				expect(errors).toHaveLength(1);
				expect(errors[0].code).toBe("INVALID_FORMAT");
			}
		});

		it("should allow empty email when not required", () => {
			const errors = Validator.validateEmail("", false);
			expect(errors).toHaveLength(0);
		});

		it("should reject empty email when required", () => {
			const errors = Validator.validateEmail("", true);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("REQUIRED");
		});
	});

	describe("validateUUID", () => {
		it("should validate correct UUID formats", () => {
			const validUUIDs = [
				"123e4567-e89b-12d3-a456-426614174000",
				"550e8400-e29b-41d4-a716-446655440000",
			];

			for (const uuid of validUUIDs) {
				const errors = Validator.validateUUID(uuid, "field");
				expect(errors).toHaveLength(0);
			}
		});

		it("should reject invalid UUID formats", () => {
			const invalidUUIDs = [
				"not-a-uuid",
				"123e4567-e89b-12d3-a456",
				"123e4567-e89b-12d3-a456-42661417400g",
				"",
			];

			for (const uuid of invalidUUIDs) {
				const errors = Validator.validateUUID(uuid, "field");
				expect(errors.length).toBeGreaterThan(0);
			}
		});
	});

	describe("validateAmount", () => {
		it("should validate positive amounts", () => {
			const errors = Validator.validateAmount(100.5, "amount");
			expect(errors).toHaveLength(0);
		});

		it("should reject negative amounts when not allowed", () => {
			const errors = Validator.validateAmount(-50, "amount", false);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("NEGATIVE_VALUE");
		});

		it("should allow negative amounts when explicitly allowed", () => {
			const errors = Validator.validateAmount(-50, "amount", true);
			expect(errors).toHaveLength(0);
		});

		it("should reject zero when not allowed", () => {
			const errors = Validator.validateAmount(0, "amount", false, false);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("ZERO_VALUE");
		});

		it("should reject invalid numbers", () => {
			const errors = Validator.validateAmount(Number.NaN, "amount");
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("INVALID_TYPE");
		});

		it("should reject amounts that are too large", () => {
			const errors = Validator.validateAmount(1e15, "amount");
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("MAX_VALUE");
		});
	});

	describe("validateDate", () => {
		it("should validate correct date formats", () => {
			const validDates = ["2023-12-25", "2024-01-01", "2023-02-28"];

			for (const date of validDates) {
				const errors = Validator.validateDate(date, "date");
				expect(errors).toHaveLength(0);
			}
		});

		it("should reject invalid date formats", () => {
			const invalidDates = [
				"2023/12/25",
				"25-12-2023",
				"2023-13-01",
				"2023-02-30",
				"invalid-date",
			];

			for (const date of invalidDates) {
				const errors = Validator.validateDate(date, "date");
				expect(errors.length).toBeGreaterThan(0);
			}
		});
	});

	describe("validatePassword", () => {
		it("should validate strong passwords", () => {
			const strongPasswords = [
				"StrongPass123!",
				"MySecure@Password1",
				"Complex#Pass2024",
			];

			for (const password of strongPasswords) {
				const errors = Validator.validatePassword(password);
				expect(errors).toHaveLength(0);
			}
		});

		it("should reject weak passwords", () => {
			const weakPasswords = [
				"weak",
				"password",
				"PASSWORD",
				"Password",
				"Password1",
				"password!",
			];

			for (const password of weakPasswords) {
				const errors = Validator.validatePassword(password);
				expect(errors.length).toBeGreaterThan(0);
			}
		});

		it("should reject passwords that are too short", () => {
			const errors = Validator.validatePassword("Short1!");
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("MIN_LENGTH");
		});

		it("should reject passwords that are too long", () => {
			const longPassword = `${"Aa1!".repeat(32)}X`; // 129 characters, has all required types
			const errors = Validator.validatePassword(longPassword);
			expect(errors).toHaveLength(1);
			expect(errors[0].code).toBe("MAX_LENGTH");
		});
	});
});

describe("SecurityUtils", () => {
	describe("sanitizeString", () => {
		it("should remove dangerous characters", () => {
			const input = '<script>alert("xss")</script>';
			const result = SecurityUtils.sanitizeString(input);
			expect(result).not.toContain("<");
			expect(result).not.toContain(">");
		});

		it("should remove javascript protocols", () => {
			const input = 'javascript:alert("xss")';
			const result = SecurityUtils.sanitizeString(input);
			expect(result).not.toContain("javascript:");
		});

		it("should remove event handlers", () => {
			const input = 'onclick=alert("xss")';
			const result = SecurityUtils.sanitizeString(input);
			expect(result).not.toContain("onclick=");
		});
	});

	describe("validateSortField", () => {
		it("should allow whitelisted fields", () => {
			const allowedFields = ["name", "created_at", "amount"];
			expect(SecurityUtils.validateSortField("name", allowedFields)).toBe(true);
		});

		it("should reject non-whitelisted fields", () => {
			const allowedFields = ["name", "created_at", "amount"];
			expect(
				SecurityUtils.validateSortField("malicious_field", allowedFields),
			).toBe(false);
		});
	});

	describe("validateSortDirection", () => {
		it("should allow valid sort directions", () => {
			expect(SecurityUtils.validateSortDirection("ASC")).toBe(true);
			expect(SecurityUtils.validateSortDirection("DESC")).toBe(true);
			expect(SecurityUtils.validateSortDirection("asc")).toBe(true);
			expect(SecurityUtils.validateSortDirection("desc")).toBe(true);
		});

		it("should reject invalid sort directions", () => {
			expect(SecurityUtils.validateSortDirection("INVALID")).toBe(false);
			expect(SecurityUtils.validateSortDirection("DROP TABLE")).toBe(false);
		});
	});

	describe("sanitizeSearchQuery", () => {
		it("should remove dangerous characters from search queries", () => {
			const input = 'search<script>alert("xss")</script>';
			const result = SecurityUtils.sanitizeSearchQuery(input);
			expect(result).not.toContain("<");
			expect(result).not.toContain(">");
			expect(result).not.toContain(";");
		});

		it("should limit search query length", () => {
			const longQuery = "a".repeat(200);
			const result = SecurityUtils.sanitizeSearchQuery(longQuery);
			expect(result.length).toBeLessThanOrEqual(100);
		});
	});
});

describe("Request Validation Functions", () => {
	describe("validateCreateUserRequest", () => {
		it("should validate valid user creation request", () => {
			const request: CreateUserRequest = {
				username: "testuser",
				email: "test@example.com",
				password: "StrongPass123!",
			};

			const result = validateCreateUserRequest(request);
			expect(result.isValid).toBe(true);
			expect(result.errors).toHaveLength(0);
		});

		it("should reject invalid user creation request", () => {
			const request: CreateUserRequest = {
				username: "ab", // Too short
				email: "invalid-email",
				password: "weak",
			};

			const result = validateCreateUserRequest(request);
			expect(result.isValid).toBe(false);
			expect(result.errors.length).toBeGreaterThan(0);
		});
	});

	describe("validateCreateAccountRequest", () => {
		it("should validate valid account creation request", () => {
			const request: CreateAccountRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				account_type_id: "123e4567-e89b-12d3-a456-426614174001",
				name: "Checking Account",
				currency: "USD",
				balance: 1000.5,
			};

			const result = validateCreateAccountRequest(request);
			expect(result.isValid).toBe(true);
			expect(result.errors).toHaveLength(0);
		});
	});

	describe("validateCreateTransactionRequest", () => {
		it("should validate valid transaction creation request", () => {
			const request: CreateTransactionRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				account_id: "123e4567-e89b-12d3-a456-426614174001",
				amount: 50.0,
				description: "Grocery shopping",
				transaction_date: new Date().toISOString(),
				transaction_type: "expense",
			};

			const result = validateCreateTransactionRequest(request);
			expect(result.isValid).toBe(true);
			expect(result.errors).toHaveLength(0);
		});
	});

	describe("validateCreateTransferRequest", () => {
		it("should validate valid transfer creation request", () => {
			const request: CreateTransferRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				from_account_id: "123e4567-e89b-12d3-a456-426614174001",
				to_account_id: "123e4567-e89b-12d3-a456-426614174002",
				amount: 100.0,
				description: "Transfer to savings",
				transfer_date: new Date().toISOString(),
			};

			const result = validateCreateTransferRequest(request);
			expect(result.isValid).toBe(true);
			expect(result.errors).toHaveLength(0);
		});

		it("should reject transfer to same account", () => {
			const request: CreateTransferRequest = {
				user_id: "123e4567-e89b-12d3-a456-426614174000",
				from_account_id: "123e4567-e89b-12d3-a456-426614174001",
				to_account_id: "123e4567-e89b-12d3-a456-426614174001", // Same as from_account_id
				amount: 100.0,
				description: "Transfer to savings",
				transfer_date: new Date().toISOString(),
			};

			const result = validateCreateTransferRequest(request);
			expect(result.isValid).toBe(false);
			expect(result.errors.some((e) => e.code === "SAME_ACCOUNT")).toBe(true);
		});
	});
});

describe("createFormValidator", () => {
	it("should create a form validator that returns field errors", () => {
		const validator = createFormValidator(validateCreateUserRequest);

		const invalidRequest: CreateUserRequest = {
			username: "ab", // Too short
			email: "invalid-email",
			password: "weak",
		};

		const result = validator(invalidRequest);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors).toHaveProperty("username");
		expect(result.fieldErrors).toHaveProperty("email");
		expect(result.fieldErrors).toHaveProperty("password");
	});
});
