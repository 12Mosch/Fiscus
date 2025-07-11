/**
 * Database Seeder Component
 * Development utility component for seeding the database with sample data
 * Only available in development mode
 */

import {
	AlertCircle,
	CheckCircle,
	CreditCard,
	Database,
	FolderOpen,
	Loader2,
	PieChart,
	Receipt,
	Target,
	Trash2,
	Users,
} from "lucide-react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import {
	SEEDING_PRESETS,
	useSeeding,
} from "@/lib/database/seeding/use-seeding";

export function DatabaseSeeder() {
	const {
		isSeeding,
		isClearing,
		error,
		lastSeeded,
		seed,
		clear,
		seedWithClear,
		clearError,
	} = useSeeding();

	// Only show in development
	if (process.env.NODE_ENV === "production") {
		return null;
	}

	const handleSeed = async (presetName: string) => {
		clearError();
		const options = SEEDING_PRESETS[presetName];
		await seed(options);
	};

	const handleClear = async () => {
		try {
			clearError();
			await clear();
		} catch (error) {
			console.error("Clearing failed:", error);
		}
	};

	const handleSeedWithClear = async (presetName: string) => {
		clearError();
		const options = SEEDING_PRESETS[presetName];
		await seedWithClear(options);
	};

	const isLoading = isSeeding || isClearing;

	return (
		<Card className="w-full max-w-4xl">
			<CardHeader>
				<div className="flex items-center gap-2">
					<Database className="h-5 w-5" />
					<CardTitle>Database Seeder</CardTitle>
					<Badge variant="secondary">Development Only</Badge>
				</div>
				<CardDescription>
					Populate the database with sample data for development and testing
				</CardDescription>
			</CardHeader>

			<CardContent className="space-y-6">
				{/* Status */}
				{error && (
					<Alert variant="destructive">
						<AlertCircle className="h-4 w-4" />
						<AlertDescription>{error}</AlertDescription>
					</Alert>
				)}

				{lastSeeded && !error && (
					<Alert>
						<CheckCircle className="h-4 w-4" />
						<AlertDescription>
							Last seeded: {lastSeeded.toLocaleString()}
						</AlertDescription>
					</Alert>
				)}

				{/* Quick Actions */}
				<div className="space-y-4">
					<h3 className="text-lg font-semibold">Quick Actions</h3>
					<div className="flex flex-wrap gap-2">
						<Button
							onClick={() => handleClear()}
							disabled={isLoading}
							variant="destructive"
							size="sm"
						>
							{isClearing ? (
								<Loader2 className="h-4 w-4 animate-spin mr-2" />
							) : (
								<Trash2 className="h-4 w-4 mr-2" />
							)}
							Clear Database
						</Button>

						<Button
							onClick={() => handleSeedWithClear("demo")}
							disabled={isLoading}
							size="sm"
						>
							{isSeeding ? (
								<Loader2 className="h-4 w-4 animate-spin mr-2" />
							) : (
								<Database className="h-4 w-4 mr-2" />
							)}
							Fresh Demo Data
						</Button>

						<Button
							onClick={() => handleSeed("minimal")}
							disabled={isLoading}
							variant="outline"
							size="sm"
						>
							{isSeeding ? (
								<Loader2 className="h-4 w-4 animate-spin mr-2" />
							) : (
								<Users className="h-4 w-4 mr-2" />
							)}
							Add Basic Data
						</Button>
					</div>
				</div>

				<Separator />

				{/* Preset Options */}
				<div className="space-y-4">
					<h3 className="text-lg font-semibold">Seeding Presets</h3>
					<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
						{Object.entries(SEEDING_PRESETS).map(([presetName, options]) => (
							<Card key={presetName} className="p-4">
								<div className="space-y-3">
									<div className="flex items-center justify-between">
										<h4 className="font-medium capitalize">{presetName}</h4>
										<Badge
											variant={
												options.clearExisting ? "destructive" : "secondary"
											}
										>
											{options.clearExisting ? "Replaces Data" : "Adds Data"}
										</Badge>
									</div>

									<div className="flex flex-wrap gap-1">
										{options.includeUsers && (
											<Badge variant="outline" className="text-xs">
												<Users className="h-3 w-3 mr-1" />
												Users
											</Badge>
										)}
										{options.includeAccounts && (
											<Badge variant="outline" className="text-xs">
												<CreditCard className="h-3 w-3 mr-1" />
												Accounts
											</Badge>
										)}
										{options.includeCategories && (
											<Badge variant="outline" className="text-xs">
												<FolderOpen className="h-3 w-3 mr-1" />
												Categories
											</Badge>
										)}
										{options.includeTransactions && (
											<Badge variant="outline" className="text-xs">
												<Receipt className="h-3 w-3 mr-1" />
												Transactions ({options.transactionsPerAccount}/account)
											</Badge>
										)}
										{options.includeBudgets && (
											<Badge variant="outline" className="text-xs">
												<PieChart className="h-3 w-3 mr-1" />
												Budgets
											</Badge>
										)}
										{options.includeGoals && (
											<Badge variant="outline" className="text-xs">
												<Target className="h-3 w-3 mr-1" />
												Goals
											</Badge>
										)}
									</div>

									<div className="flex gap-2">
										<Button
											onClick={() => handleSeed(presetName)}
											disabled={isLoading}
											size="sm"
											variant="outline"
											className="flex-1"
										>
											{isSeeding ? (
												<Loader2 className="h-4 w-4 animate-spin mr-2" />
											) : (
												<Database className="h-4 w-4 mr-2" />
											)}
											Seed
										</Button>

										{!options.clearExisting && (
											<Button
												onClick={() => handleSeedWithClear(presetName)}
												disabled={isLoading}
												size="sm"
												variant="destructive"
												className="flex-1"
											>
												{isSeeding ? (
													<Loader2 className="h-4 w-4 animate-spin mr-2" />
												) : (
													<Trash2 className="h-4 w-4 mr-2" />
												)}
												Replace
											</Button>
										)}
									</div>
								</div>
							</Card>
						))}
					</div>
				</div>

				{/* Warning */}
				<Alert>
					<AlertCircle className="h-4 w-4" />
					<AlertDescription>
						<strong>Warning:</strong> Seeding operations will modify your
						database. Use "Clear Database" or "Replace" options carefully as
						they will permanently delete existing data.
					</AlertDescription>
				</Alert>
			</CardContent>
		</Card>
	);
}
