import { describe, expect, it } from "vitest";

import { decodeFromBase64, encodeToBase64 } from "../encryption";

describe("Base64 Encoding/Decoding", () => {
	it("should encode and decode simple ASCII strings correctly", () => {
		const testString = "Hello, World!";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should handle UTF-8 characters correctly", () => {
		const testString = "Hello 世界! 🌍 Café naïve résumé";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should handle empty strings", () => {
		const testString = "";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should handle special characters and symbols", () => {
		const testString = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should handle newlines and whitespace", () => {
		const testString = "Line 1\nLine 2\r\nLine 3\tTabbed\n  Spaced  ";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should handle financial data formats", () => {
		const testString = "$12,345.67 - Salary payment from ACME Corp";
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
	});

	it("should produce valid base64 output", () => {
		const testString = "Test string for base64 validation";
		const encoded = encodeToBase64(testString);

		// Base64 should only contain valid characters
		expect(encoded).toMatch(/^[A-Za-z0-9+/]*={0,2}$/);
	});

	it("should be compatible with standard base64 for ASCII", () => {
		const testString = "Simple ASCII test";
		const ourEncoded = encodeToBase64(testString);

		// For ASCII strings, our encoding should match btoa
		const standardEncoded = btoa(testString);
		expect(ourEncoded).toBe(standardEncoded);

		// And our decoder should handle standard base64
		const ourDecoded = decodeFromBase64(standardEncoded);
		expect(ourDecoded).toBe(testString);
	});

	it("should handle long strings", () => {
		const testString = "A".repeat(10000);
		const encoded = encodeToBase64(testString);
		const decoded = decodeFromBase64(encoded);

		expect(decoded).toBe(testString);
		expect(decoded.length).toBe(10000);
	});

	it("should handle various Unicode ranges", () => {
		const testCases = [
			"Latin: àáâãäåæçèéêë",
			"Cyrillic: абвгдеёжзийклмнопрстуфхцчшщъыьэюя",
			"Greek: αβγδεζηθικλμνξοπρστυφχψω",
			"Arabic: العربية",
			"Chinese: 中文测试",
			"Japanese: ひらがな カタカナ 漢字",
			"Emoji: 😀😃😄😁😆😅😂🤣",
			"Math symbols: ∑∏∫∂∇∆√∞≠≤≥±×÷",
		];

		for (const testString of testCases) {
			const encoded = encodeToBase64(testString);
			const decoded = decodeFromBase64(encoded);
			expect(decoded).toBe(testString);
		}
	});
});
