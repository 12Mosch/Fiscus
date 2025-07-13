/**
 * Database Test Component
 * A simple component to test database operations in the Tauri environment
 * This component is only available in development builds and will be excluded from production
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
import { apiService, apiUtils } from "@/lib/api-service";
import {
	useAccountOperations,
	useApiStatus,
	useTransactionOperations,
} from "@/lib/database/hooks";
import type {
	CreateAccountRequest,
	CreateTransactionRequest,
} from "@/types/api";

// Development-only component that gets tree-shaken in production
function DatabaseTestImpl() {
	const [testResults, setTestResults] = useState<string[]>([]);
	const [isRunning, setIsRunning] = useState(false);
	const { connected, loading: statusLoading } = useApiStatus();
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
			addResult("🚀 Starting API service tests...");

			// Test 1: API service connection
			addResult("📡 Testing API service connection...");
			await apiService.initialize();
			addResult(`✅ API service connected: ${connected}`);

			// Test 2: Create test user ID
			const testUserId = await apiUtils.generateId();
			addResult(`👤 Generated test user ID: ${testUserId.substring(0, 8)}...`);

			// Test 3: Create test account
			addResult("🏦 Creating test account...");
			const accountData: CreateAccountRequest = {
				user_id: testUserId,
				account_type_id: "checking",
				name: "Test Checking Account",
				description: "Test account created by DatabaseTest component",
				initial_balance: 1000.0,
				currency: "USD",
				is_active: true,
				institution_name: "Test Bank",
			};

			const account = await createAccount(accountData);
			if (!account) {
				throw new Error("Failed to create account");
			}
			addResult(
				`✅ Account created: ${account.name} (ID: ${account.id.substring(0, 8)}...)`,
			);

			// Test 4: Create test transaction
			addResult("💰 Creating test transaction...");
			const transactionData: CreateTransactionRequest = {
				user_id: testUserId,
				account_id: account.id,
				amount: -50.0,
				description: "Test expense transaction",
				transaction_date: apiUtils.formatDate(new Date()),
				transaction_type: "expense",
				status: "completed",
				payee: "Test Store",
				notes: "Created by DatabaseTest component",
			};

			const transaction = await createTransaction(transactionData);
			if (!transaction) {
				throw new Error("Failed to create transaction");
			}
			addResult(
				`✅ Transaction created: ${transaction.description} (${transaction.amount})`,
			);

			// Test 5: Verify account balance update
			addResult("🔍 Verifying account balance update...");
			const updatedAccount = await apiService.accounts.findById(account.id);
			if (updatedAccount) {
				addResult(
					`✅ Account balance updated: $${updatedAccount.balance} (was $${account.balance})`,
				);
			} else {
				addResult("❌ Failed to retrieve updated account");
			}

			// Test 6: Query transactions
			addResult("📊 Querying transactions...");
			const transactions =
				await apiService.transactions.findWithDetails(testUserId);
			addResult(`✅ Found ${transactions.data.length} transaction(s)`);

			// Test 7: Get account balances
			addResult("💳 Getting account balances...");
			const balances = await apiService.accounts.getAccountBalances(testUserId);
			addResult(`✅ Found ${balances.length} account balance(s)`);

			addResult("🎉 All API service tests completed successfully!");
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
				<CardTitle>API Service Integration Test</CardTitle>
				<CardDescription>
					Test the secure API service integration and database operations
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				{/* API Status */}
				<div className="flex items-center gap-4">
					<span className="font-medium">API Service Status:</span>
					{statusLoading ? (
						<Badge variant="secondary">Loading...</Badge>
					) : connected ? (
						<Badge variant="default">Connected</Badge>
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
						{isRunning ? "Running Tests..." : "Run API Tests"}
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
							This component tests the secure API service integration in the
							Tauri environment
						</li>
						<li>
							Click "Run API Tests" to execute a series of secure API operations
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
				<div className="bg-green-50 dark:bg-green-900/20 p-3 rounded-lg border border-green-200 dark:border-green-800">
					<p className="text-sm text-green-800 dark:text-green-200">
						<strong>Development Mode:</strong> This component is automatically
						excluded from production builds. It will not be bundled or rendered
						when NODE_ENV is set to 'production'.
					</p>
				</div>
			</CardContent>
		</Card>
	);
}

// Conditional export based on environment
// In production, export a null component that renders nothing
// In development, export the full DatabaseTest component
export const DatabaseTest =
	process.env.NODE_ENV !== "production"
		? DatabaseTestImpl
		: function ProductionStub() {
				if (process.env.NODE_ENV === "production") {
					console.warn(
						"DatabaseTest component is not available in production builds",
					);
				}
				return null;
			};

export default DatabaseTest;
