/**
 * Tests for input validation in repository base class
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BaseEntity } from "../../types";
import { BaseRepository } from "../base";

// Mock entity types for testing
interface TestEntity extends BaseEntity {
	id: string;
	name: string;
	description?: string;
	value: number;
	created_at: string;
	updated_at?: string;
}

interface TestCreateInput extends Record<string, unknown> {
	name: string;
	description?: string;
	value: number;
	malicious_field?: string;
}

interface TestUpdateInput extends Record<string, unknown> {
	name?: string;
	description?: string;
	value?: number;
	another_malicious_field?: string;
}

// Test implementation of BaseRepository
class TestRepository extends BaseRepository<
	TestEntity,
	TestCreateInput,
	TestUpdateInput
> {
	protected tableName = "test_table";
	protected selectFields =
		"id, name, description, value, created_at, updated_at";

	protected getAllowedSortFields(): string[] {
		return ["id", "name", "value", "created_at", "updated_at"];
	}

	protected getAllowedCreateFields(): string[] {
		return ["name", "description", "value"];
	}

	protected getAllowedUpdateFields(): string[] {
		return ["name", "description", "value"];
	}

	// Expose protected methods for testing
	public testValidateInputFields<T extends Record<string, unknown>>(
		input: T,
		allowedFields: string[],
		operation: string,
	) {
		return this.validateInputFields(input, allowedFields, operation);
	}

	public testGetAllowedCreateFields() {
		return this.getAllowedCreateFields();
	}

	public testGetAllowedUpdateFields() {
		return this.getAllowedUpdateFields();
	}
}

describe("Input Validation Tests", () => {
	let repository: TestRepository;
	let consoleSpy: ReturnType<typeof vi.spyOn>;

	beforeEach(() => {
		repository = new TestRepository();
		consoleSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
	});

	afterEach(() => {
		consoleSpy.mockRestore();
	});

	describe("validateInputFields", () => {
		it("should allow valid fields", () => {
			const input = {
				name: "Test Name",
				description: "Test Description",
				value: 100,
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "description", "value"],
				"test",
			);

			expect(result).toEqual(input);
			expect(consoleSpy).not.toHaveBeenCalled();
		});

		it("should filter out invalid fields", () => {
			const input = {
				name: "Test Name",
				description: "Test Description",
				value: 100,
				malicious_field: "DROP TABLE users;",
				another_invalid: "SELECT * FROM passwords;",
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "description", "value"],
				"test",
			);

			expect(result).toEqual({
				name: "Test Name",
				description: "Test Description",
				value: 100,
			});
			expect(consoleSpy).toHaveBeenCalledWith(
				"Invalid fields attempted in test operation: malicious_field, another_invalid. " +
					"Allowed fields: name, description, value",
			);
		});

		it("should handle empty input", () => {
			const input = {};

			const result = repository.testValidateInputFields(
				input,
				["name", "description", "value"],
				"test",
			);

			expect(result).toEqual({});
			expect(consoleSpy).not.toHaveBeenCalled();
		});

		it("should handle input with only invalid fields", () => {
			const input = {
				malicious_field: "DROP TABLE users;",
				another_invalid: "SELECT * FROM passwords;",
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "description", "value"],
				"test",
			);

			expect(result).toEqual({});
			expect(consoleSpy).toHaveBeenCalledWith(
				"Invalid fields attempted in test operation: malicious_field, another_invalid. " +
					"Allowed fields: name, description, value",
			);
		});

		it("should preserve data types", () => {
			const input = {
				name: "Test Name",
				value: 42,
				is_active: true,
				tags: ["tag1", "tag2"],
				metadata: { key: "value" },
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "value", "is_active", "tags", "metadata"],
				"test",
			);

			expect(result).toEqual(input);
			expect(typeof result.name).toBe("string");
			expect(typeof result.value).toBe("number");
			expect(typeof result.is_active).toBe("boolean");
			expect(Array.isArray(result.tags)).toBe(true);
			expect(typeof result.metadata).toBe("object");
		});

		it("should handle partial matches", () => {
			const input = {
				name: "Test Name",
				description: "Test Description",
				value: 100,
				invalid_field: "malicious",
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "value"], // description not allowed
				"test",
			);

			expect(result).toEqual({
				name: "Test Name",
				value: 100,
			});
			expect(consoleSpy).toHaveBeenCalledWith(
				"Invalid fields attempted in test operation: description, invalid_field. " +
					"Allowed fields: name, value",
			);
		});
	});

	describe("getAllowedCreateFields", () => {
		it("should return correct create fields", () => {
			const fields = repository.testGetAllowedCreateFields();
			expect(fields).toEqual(["name", "description", "value"]);
		});
	});

	describe("getAllowedUpdateFields", () => {
		it("should return correct update fields", () => {
			const fields = repository.testGetAllowedUpdateFields();
			expect(fields).toEqual(["name", "description", "value"]);
		});
	});

	describe("field validation integration", () => {
		it("should prevent SQL injection attempts in field names", () => {
			const maliciousInput = {
				"name; DROP TABLE users; --": "Test Name",
				"description' OR '1'='1": "Test Description",
				value: 100,
			};

			const result = repository.testValidateInputFields(
				maliciousInput,
				["name", "description", "value"],
				"test",
			);

			// Should filter out malicious fields but keep valid ones
			expect(result).toEqual({
				value: 100,
			});
			expect(consoleSpy).toHaveBeenCalledWith(
				expect.stringContaining("Invalid fields attempted in test operation"),
			);
		});

		it("should handle case sensitivity correctly", () => {
			const input = {
				Name: "Test Name", // Capital N
				DESCRIPTION: "Test Description", // All caps
				value: 100,
			};

			const result = repository.testValidateInputFields(
				input,
				["name", "description", "value"], // lowercase
				"test",
			);

			// Should filter out case-mismatched fields
			expect(result).toEqual({
				value: 100,
			});
			expect(consoleSpy).toHaveBeenCalledWith(
				"Invalid fields attempted in test operation: Name, DESCRIPTION. " +
					"Allowed fields: name, description, value",
			);
		});
	});
});
