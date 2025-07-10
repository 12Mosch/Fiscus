/**
 * Development Tools Page
 * This page provides access to development utilities like database seeding
 * Only available in development mode
 */

import { createFileRoute } from "@tanstack/react-router";
import { Code, Database, Settings, TestTube } from "lucide-react";
import { DatabaseSeeder } from "@/components/debug/DatabaseSeeder";
import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export const Route = createFileRoute("/dev")({
	component: DevelopmentPage,
});

function DevelopmentPage() {
	// Only show in development
	if (process.env.NODE_ENV === "production") {
		return (
			<div className="flex items-center justify-center min-h-screen">
				<Card className="w-full max-w-md">
					<CardHeader className="text-center">
						<CardTitle className="flex items-center justify-center gap-2">
							<Settings className="h-5 w-5" />
							Development Tools
						</CardTitle>
						<CardDescription>
							This page is only available in development mode
						</CardDescription>
					</CardHeader>
				</Card>
			</div>
		);
	}

	return (
		<div className="container mx-auto py-8 space-y-8">
			{/* Header */}
			<div className="space-y-2">
				<div className="flex items-center gap-2">
					<Code className="h-6 w-6" />
					<h1 className="text-3xl font-bold">Development Tools</h1>
					<Badge variant="secondary">Development Only</Badge>
				</div>
				<p className="text-muted-foreground">
					Utilities and tools for development and testing of the Fiscus
					application
				</p>
			</div>

			<Separator />

			{/* Database Tools Section */}
			<div className="space-y-6">
				<div className="flex items-center gap-2">
					<Database className="h-5 w-5" />
					<h2 className="text-2xl font-semibold">Database Tools</h2>
				</div>

				<DatabaseSeeder />
			</div>

			<Separator />

			{/* Additional Development Tools */}
			<div className="space-y-6">
				<div className="flex items-center gap-2">
					<TestTube className="h-5 w-5" />
					<h2 className="text-2xl font-semibold">Testing & Debugging</h2>
				</div>

				<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					<Card>
						<CardHeader>
							<CardTitle className="text-lg">Environment Info</CardTitle>
							<CardDescription>Current environment details</CardDescription>
						</CardHeader>
						<CardContent className="space-y-2">
							<div className="flex justify-between">
								<span className="font-medium">Node Environment:</span>
								<Badge variant="outline">{process.env.NODE_ENV}</Badge>
							</div>
							<div className="flex justify-between">
								<span className="font-medium">Development Mode:</span>
								<Badge
									variant={
										process.env.NODE_ENV === "development"
											? "default"
											: "secondary"
									}
								>
									{process.env.NODE_ENV === "development"
										? "Enabled"
										: "Disabled"}
								</Badge>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle className="text-lg">Database Status</CardTitle>
							<CardDescription>Database connection information</CardDescription>
						</CardHeader>
						<CardContent className="space-y-2">
							<div className="flex justify-between">
								<span className="font-medium">Database Type:</span>
								<Badge variant="outline">SQLite</Badge>
							</div>
							<div className="flex justify-between">
								<span className="font-medium">Database File:</span>
								<Badge variant="outline">fiscus.db</Badge>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle className="text-lg">Quick Actions</CardTitle>
							<CardDescription>Common development tasks</CardDescription>
						</CardHeader>
						<CardContent className="space-y-2">
							<p className="text-sm text-muted-foreground">
								Use the Database Seeder above for data management tasks.
							</p>
							<p className="text-sm text-muted-foreground">
								Additional tools can be added here as needed.
							</p>
						</CardContent>
					</Card>
				</div>
			</div>

			{/* Usage Instructions */}
			<div className="space-y-4">
				<h3 className="text-xl font-semibold">Usage Instructions</h3>
				<Card>
					<CardContent className="pt-6">
						<div className="space-y-4">
							<div>
								<h4 className="font-medium mb-2">Database Seeding</h4>
								<ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
									<li>
										Use "Fresh Demo Data" to clear and populate with demo data
									</li>
									<li>
										Use "Add Basic Data" to add minimal data without clearing
									</li>
									<li>Use preset options for specific seeding scenarios</li>
									<li>
										Always backup important data before clearing operations
									</li>
								</ul>
							</div>

							<div>
								<h4 className="font-medium mb-2">Command Line Seeding</h4>
								<div className="bg-muted p-3 rounded-md">
									<code className="text-sm">
										npm run seed # Default seeding
										<br />
										npm run seed:clear # Clear and seed
										<br />
										npm run seed:minimal # Basic data only
										<br />
										npm run seed:demo # Demo configuration
									</code>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	);
}
