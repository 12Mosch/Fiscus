/**
 * Tests for AccountCard component
 */

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Account } from "@/types/dashboard";
import { AccountCard } from "../AccountCard";

describe("AccountCard", () => {
	const baseAccount: Account = {
		id: "test-1",
		name: "Test Account",
		type: "checking",
		balance: 1000,
		currency: "USD",
		lastUpdated: new Date("2024-01-15T10:30:00Z"),
		accountNumber: "****1234",
	};

	it("renders account information correctly", () => {
		render(<AccountCard account={baseAccount} />);

		expect(screen.getByText("Test Account")).toBeInTheDocument();
		expect(screen.getByText("Checking")).toBeInTheDocument();
		expect(screen.getByText("****1234")).toBeInTheDocument();
		expect(screen.getByText("$1,000.00")).toBeInTheDocument();
	});

	it("displays available credit correctly for credit accounts with credit limit", () => {
		const creditAccount: Account = {
			...baseAccount,
			name: "Credit Card",
			type: "credit",
			balance: -1250.75, // Debt of $1,250.75
			creditLimit: 5000, // Credit limit of $5,000
		};

		render(<AccountCard account={creditAccount} />);

		// Should show the debt as negative balance
		expect(screen.getByText("-$1,250.75")).toBeInTheDocument();

		// Should show available credit: $5,000 - $1,250.75 = $3,749.25
		expect(screen.getByText("Available Credit")).toBeInTheDocument();
		expect(screen.getByText("$3,749.25")).toBeInTheDocument();

		// Should show credit limit
		expect(screen.getByText("Credit Limit")).toBeInTheDocument();
		expect(screen.getByText("$5,000.00")).toBeInTheDocument();
	});

	it("handles credit account with positive balance (credit)", () => {
		const creditAccount: Account = {
			...baseAccount,
			name: "Credit Card",
			type: "credit",
			balance: 100, // Credit balance of $100
			creditLimit: 5000,
		};

		render(<AccountCard account={creditAccount} />);

		// Should show positive balance
		expect(screen.getByText("$100.00")).toBeInTheDocument();

		// Available credit should be full limit since no debt
		expect(screen.getByText("Available Credit")).toBeInTheDocument();

		// Check for both available credit and credit limit values
		const creditValues = screen.getAllByText("$5,000.00");
		expect(creditValues).toHaveLength(2); // Available credit and credit limit both show $5,000

		// Verify credit limit label exists
		expect(screen.getByText("Credit Limit")).toBeInTheDocument();
	});

	it("does not show credit information for credit accounts without credit limit", () => {
		const creditAccount: Account = {
			...baseAccount,
			name: "Credit Card",
			type: "credit",
			balance: -1250.75,
			// No creditLimit provided
		};

		render(<AccountCard account={creditAccount} />);

		// Should show the debt as negative balance
		expect(screen.getByText("-$1,250.75")).toBeInTheDocument();

		// Should NOT show available credit section
		expect(screen.queryByText("Available Credit")).not.toBeInTheDocument();
		expect(screen.queryByText("Credit Limit")).not.toBeInTheDocument();
	});

	it("does not show credit information for non-credit accounts", () => {
		const savingsAccount: Account = {
			...baseAccount,
			name: "Savings Account",
			type: "savings",
			balance: 5000,
		};

		render(<AccountCard account={savingsAccount} />);

		expect(screen.getByText("$5,000.00")).toBeInTheDocument();
		expect(screen.queryByText("Available Credit")).not.toBeInTheDocument();
		expect(screen.queryByText("Credit Limit")).not.toBeInTheDocument();
	});

	it("shows investment portfolio value for investment accounts", () => {
		const investmentAccount: Account = {
			...baseAccount,
			name: "Investment Portfolio",
			type: "investment",
			balance: 42350.25,
		};

		render(<AccountCard account={investmentAccount} />);

		// Should show balance in both main section and portfolio section
		const balanceValues = screen.getAllByText("$42,350.25");
		expect(balanceValues).toHaveLength(2); // Main balance and portfolio value

		expect(screen.getByText("Portfolio Value")).toBeInTheDocument();
	});

	it("applies correct balance colors for credit accounts", () => {
		const creditAccountWithDebt: Account = {
			...baseAccount,
			name: "Credit Card",
			type: "credit",
			balance: -1250.75,
			creditLimit: 5000,
		};

		const { rerender } = render(
			<AccountCard account={creditAccountWithDebt} />,
		);

		// Debt should be shown in red
		const debtElement = screen.getByText("-$1,250.75");
		expect(debtElement).toHaveClass("text-red-600");

		// Test positive balance (credit) - should be green
		const creditAccountWithCredit: Account = {
			...creditAccountWithDebt,
			balance: 100,
		};

		rerender(<AccountCard account={creditAccountWithCredit} />);
		const creditElement = screen.getByText("$100.00");
		expect(creditElement).toHaveClass("text-green-600");
	});
});
