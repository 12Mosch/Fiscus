/**
 * Tests for DatabaseTest component conditional rendering
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

describe("DatabaseTest Component", () => {
	let originalNodeEnv: string | undefined;

	beforeEach(() => {
		originalNodeEnv = process.env.NODE_ENV;
		// Clear module cache to ensure fresh imports
		vi.resetModules();
	});

	afterEach(() => {
		process.env.NODE_ENV = originalNodeEnv;
	});

	it("should export the full component in development mode", async () => {
		process.env.NODE_ENV = "development";

		const { DatabaseTest } = await import("../DatabaseTest");

		expect(DatabaseTest).toBeDefined();
		expect(typeof DatabaseTest).toBe("function");
		expect(DatabaseTest.name).toBe("DatabaseTestImpl");
	});

	it("should export a stub component in production mode", async () => {
		process.env.NODE_ENV = "production";

		const consoleSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

		const { DatabaseTest } = await import("../DatabaseTest");

		expect(DatabaseTest).toBeDefined();
		expect(typeof DatabaseTest).toBe("function");
		expect(DatabaseTest.name).toBe("ProductionStub");

		// Test that the production stub returns null
		const result = DatabaseTest();
		expect(result).toBeNull();

		// Test that it logs a warning
		expect(consoleSpy).toHaveBeenCalledWith(
			"DatabaseTest component is not available in production builds",
		);

		consoleSpy.mockRestore();
	});

	it("should export the full component when NODE_ENV is not set", async () => {
		delete process.env.NODE_ENV;

		const { DatabaseTest } = await import("../DatabaseTest");

		expect(DatabaseTest).toBeDefined();
		expect(typeof DatabaseTest).toBe("function");
		expect(DatabaseTest.name).toBe("DatabaseTestImpl");
	});

	it("should export the full component in test environment", async () => {
		process.env.NODE_ENV = "test";

		const { DatabaseTest } = await import("../DatabaseTest");

		expect(DatabaseTest).toBeDefined();
		expect(typeof DatabaseTest).toBe("function");
		expect(DatabaseTest.name).toBe("DatabaseTestImpl");
	});
});
