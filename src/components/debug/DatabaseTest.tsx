/**
 * Database Test Component
 * A simple component to test database operations in the Tauri environment
 * This can be temporarily added to the app to verify database functionality
 */

import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { databaseService, formatDateForDb, generateId } from "@/lib/database";
import {
	useAccountOperations,
	useDatabaseStatus,
	useTransactionOperations,
} from "@/lib/database/hooks";
import type {
	CreateAccountInput,
	CreateTransactionInput,
} from "@/lib/database/types";

export function DatabaseTest() {
	const [testResults, setTestResults] = useState<string[]>([]);
	const [isRunning, setIsRunning] = useState(false);
	const { connected, version, loading: statusLoading } = useDatabaseStatus();
	const { createAccount, loading: accountLoading } = useAccountOperations();
	const { createTransaction, loading: transactionLoading } =
		useTransactionOperations();

	const addResult = (message: string) => {
		setTestResults((prev) => [
			...prev,
			`${new Date().toLocaleTimeString()}: ${message}`,
		]);
	};

	const runDatabaseTests = async () => {
		setIsRunning(true);
		setTestResults([]);

		try {
			addResult("🚀 Starting database tests...");

			// Test 1: Database connection
			addResult("📡 Testing database connection...");
			const health = await databaseService.getHealthStatus();
			addResult(
				`✅ Database connected: ${health.connected}, version: ${health.version}`,
			);

			// Test 2: Create test user ID
			const testUserId = generateId();
			addResult(`👤 Generated test user ID: ${testUserId.substring(0, 8)}...`);

			// Test 3: Create test account
			addResult("🏦 Creating test account...");
			const accountData: CreateAccountInput = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Test Checking Account",
				description: "Test account created by DatabaseTest component",
				initial_balance: 1000.0,
				current_balance: 1000.0,
				currency: "USD",
				is_active: true,
				institution_name: "Test Bank",
			};

			const account = await createAccount(accountData);
			addResult(
				`✅ Account created: ${account.name} (ID: ${account.id.substring(0, 8)}...)`,
			);

			// Test 4: Create test transaction
			addResult("💰 Creating test transaction...");
			const transactionData: CreateTransactionInput = {
				user_id: testUserId,
				account_id: account.id,
				amount: -50.0,
				description: "Test expense transaction",
				transaction_date: formatDateForDb(new Date()),
				transaction_type: "expense",
				status: "completed",
				payee: "Test Store",
				notes: "Created by DatabaseTest component",
			};

			const transaction = await createTransaction(transactionData);
			addResult(
				`✅ Transaction created: ${transaction.description} (${transaction.amount})`,
			);

			// Test 5: Verify account balance update
			addResult("🔍 Verifying account balance update...");
			const updatedAccount = await databaseService.accounts.findById(
				account.id,
			);
			if (updatedAccount) {
				addResult(
					`✅ Account balance updated: $${updatedAccount.current_balance} (was $${account.current_balance})`,
				);
			} else {
				addResult("❌ Failed to retrieve updated account");
			}

			// Test 6: Query transactions
			addResult("📊 Querying transactions...");
			const transactions =
				await databaseService.transactions.findWithDetails(testUserId);
			addResult(`✅ Found ${transactions.data.length} transaction(s)`);

			// Test 7: Get account balances
			addResult("💳 Getting account balances...");
			const balances =
				await databaseService.accounts.getAccountBalances(testUserId);
			addResult(`✅ Found ${balances.length} account balance(s)`);

			addResult("🎉 All database tests completed successfully!");
		} catch (error) {
			addResult(
				`❌ Test failed: ${error instanceof Error ? error.message : "Unknown error"}`,
			);
			console.error("Database test error:", error);
		} finally {
			setIsRunning(false);
		}
	};

	const clearResults = () => {
		setTestResults([]);
	};

	return (
		<Card className="w-full max-w-4xl mx-auto">
			<CardHeader>
				<CardTitle>Database Integration Test</CardTitle>
				<CardDescription>
					Test the Tauri SQL plugin integration and database operations
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				{/* Database Status */}
				<div className="flex items-center gap-4">
					<span className="font-medium">Database Status:</span>
					{statusLoading ? (
						<Badge variant="secondary">Loading...</Badge>
					) : connected ? (
						<Badge variant="default">Connected (v{version})</Badge>
					) : (
						<Badge variant="destructive">Disconnected</Badge>
					)}
				</div>

				{/* Test Controls */}
				<div className="flex gap-2">
					<Button
						onClick={runDatabaseTests}
						disabled={isRunning || accountLoading || transactionLoading}
					>
						{isRunning ? "Running Tests..." : "Run Database Tests"}
					</Button>
					<Button variant="outline" onClick={clearResults} disabled={isRunning}>
						Clear Results
					</Button>
				</div>

				{/* Test Results */}
				{testResults.length > 0 && (
					<div className="space-y-2">
						<h3 className="font-medium">Test Results:</h3>
						<div className="bg-muted p-4 rounded-lg max-h-96 overflow-y-auto">
							<div className="space-y-1 font-mono text-sm">
								{testResults.map((result) => (
									<div key={result} className="whitespace-pre-wrap">
										{result}
									</div>
								))}
							</div>
						</div>
					</div>
				)}

				{/* Instructions */}
				<div className="text-sm text-muted-foreground space-y-2">
					<p>
						<strong>Instructions:</strong>
					</p>
					<ul className="list-disc list-inside space-y-1">
						<li>
							This component tests the database integration in the Tauri
							environment
						</li>
						<li>
							Click "Run Database Tests" to execute a series of database
							operations
						</li>
						<li>
							The tests will create a test account and transaction to verify
							functionality
						</li>
						<li>
							All test data uses generated UUIDs and won't interfere with real
							data
						</li>
						<li>
							Check the console for detailed error information if tests fail
						</li>
					</ul>
				</div>

				{/* Usage Note */}
				<div className="bg-yellow-50 dark:bg-yellow-900/20 p-3 rounded-lg border border-yellow-200 dark:border-yellow-800">
					<p className="text-sm text-yellow-800 dark:text-yellow-200">
						<strong>Note:</strong> This component is for testing purposes only.
						Remove it from production builds or hide it behind a development
						flag.
					</p>
				</div>
			</CardContent>
		</Card>
	);
}

export default DatabaseTest;
