#!/usr/bin/env node

/**
 * Database seeding script for development
 * This script can be run to populate the database with sample data
 *
 * Usage:
 *   npm run seed              # Seed with default options
 *   npm run seed:clear        # Clear database and seed with fresh data
 *   npm run seed:minimal      # Seed with minimal data (users, accounts, categories only)
 */

import { clearDatabase, type SeedOptions, seedDatabase } from "./index";

// Parse command line arguments
const args = process.argv.slice(2);
const command = args[0] || "default";

async function runSeeding() {
	try {
		let options: SeedOptions;

		switch (command) {
			case "clear":
				console.log("🧹 Running seed with clear existing data...");
				options = {
					clearExisting: true,
					includeUsers: true,
					includeAccounts: true,
					includeCategories: true,
					includeTransactions: true,
					includeBudgets: true,
					includeGoals: true,
					transactionsPerAccount: 25,
				};
				break;

			case "minimal":
				console.log("🌱 Running minimal seed...");
				options = {
					clearExisting: false,
					includeUsers: true,
					includeAccounts: true,
					includeCategories: true,
					includeTransactions: false,
					includeBudgets: false,
					includeGoals: false,
				};
				break;

			case "transactions-only":
				console.log("💳 Seeding transactions only...");
				options = {
					clearExisting: false,
					includeUsers: false,
					includeAccounts: false,
					includeCategories: false,
					includeTransactions: true,
					includeBudgets: false,
					includeGoals: false,
					transactionsPerAccount: 30,
				};
				break;

			case "clear-only":
				console.log("🧹 Clearing database only...");
				await clearDatabase();
				console.log("✅ Database cleared successfully!");
				return;

			default:
				console.log("🌱 Running default seed...");
				options = {
					clearExisting: false,
					includeUsers: true,
					includeAccounts: true,
					includeCategories: true,
					includeTransactions: true,
					includeBudgets: true,
					includeGoals: true,
					transactionsPerAccount: 20,
				};
				break;
		}

		await seedDatabase(options);
		console.log("🎉 Seeding completed successfully!");
	} catch (error) {
		console.error("❌ Seeding failed:", error);
		process.exit(1);
	}
}

// Handle graceful shutdown
process.on("SIGINT", () => {
	console.log("\n⚠️  Seeding interrupted by user");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("\n⚠️  Seeding terminated");
	process.exit(0);
});

// Run the seeding
runSeeding().catch((error) => {
	console.error("❌ Unexpected error:", error);
	process.exit(1);
});
