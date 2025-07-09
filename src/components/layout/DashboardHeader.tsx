/**
 * Dashboard Header Component
 * Top navigation bar with user menu, notifications, and mobile menu toggle
 */

import { Bell, LogOut, Menu, Search, Settings, User } from "lucide-react";
import { useState } from "react";
import { ModeToggle } from "@/components/mode-toggle";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import {
	Sheet,
	SheetContent,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "@/components/ui/sheet";
import { mockNotifications } from "@/data/mockData";
import type { NotificationItem } from "@/types/dashboard";

interface DashboardHeaderProps {
	onMenuClick?: () => void;
}

export function DashboardHeader({ onMenuClick }: DashboardHeaderProps) {
	const [searchQuery, setSearchQuery] = useState("");
	const [isMobileSearchOpen, setIsMobileSearchOpen] = useState(false);

	// Mock user data
	const user = {
		name: "John Doe",
		email: "john.doe@example.com",
		avatar: undefined,
	};

	const unreadNotifications = mockNotifications.filter((n) => !n.read);

	const handleSearch = (query: string) => {
		// TODO: Implement actual search functionality
		// This could filter transactions, accounts, etc.
		console.log("Searching for:", query);
	};

	const handleMobileSearchToggle = () => {
		setIsMobileSearchOpen(!isMobileSearchOpen);
	};

	const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const query = e.target.value;
		setSearchQuery(query);
		// Debounce search or trigger on enter
		if (query.length > 2 || query.length === 0) {
			handleSearch(query);
		}
	};

	const handleNotificationClick = (notification: NotificationItem) => {
		console.log("Notification clicked:", notification);
		// TODO: Implement notification handling
		// In a real app, this would mark as read and potentially navigate
	};

	const getNotificationIcon = (type: NotificationItem["type"]) => {
		const iconClass = "h-4 w-4";
		switch (type) {
			case "error":
				return <div className={`${iconClass} text-red-500`}>⚠</div>;
			case "warning":
				return <div className={`${iconClass} text-yellow-500`}>⚠</div>;
			case "success":
				return <div className={`${iconClass} text-green-500`}>✓</div>;
			default:
				return <div className={`${iconClass} text-blue-500`}>ℹ</div>;
		}
	};

	return (
		<header className="sticky top-0 z-40 border-b border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-950">
			<div className="flex h-16 items-center justify-between px-6">
				{/* Left Section */}
				<div className="flex items-center gap-4">
					{/* Mobile Menu Button */}
					<Button
						variant="ghost"
						size="sm"
						className="lg:hidden"
						onClick={onMenuClick}
						aria-label="Toggle menu"
					>
						<Menu className="h-5 w-5" />
					</Button>

					{/* Search */}
					<div className="relative hidden md:block">
						<Search className="absolute left-1 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400 z-10 pointer-events-none" />
						<Input
							placeholder="Search transactions, accounts..."
							value={searchQuery}
							onChange={handleSearchChange}
							className="w-64 pl-10"
						/>
					</div>
				</div>

				{/* Right Section */}
				<div className="flex items-center gap-2">
					{/* Search Button (Mobile) */}
					<Button
						variant="ghost"
						size="sm"
						className="md:hidden"
						onClick={handleMobileSearchToggle}
						aria-label="Toggle search"
					>
						<Search className="h-5 w-5" />
					</Button>

					{/* Theme Toggle */}
					<ModeToggle />

					{/* Notifications */}
					<Sheet>
						<SheetTrigger asChild>
							<Button variant="ghost" size="sm" className="relative">
								<Bell className="h-5 w-5" />
								{unreadNotifications.length > 0 && (
									<Badge
										variant="destructive"
										className="absolute -right-1 -top-1 h-5 w-5 rounded-full p-0 text-xs"
									>
										{unreadNotifications.length}
									</Badge>
								)}
							</Button>
						</SheetTrigger>
						<SheetContent>
							<SheetHeader>
								<SheetTitle>Notifications</SheetTitle>
							</SheetHeader>
							<div className="mt-6 space-y-4">
								{mockNotifications.length === 0 ? (
									<p className="text-center text-gray-500">No notifications</p>
								) : (
									mockNotifications.map((notification) => (
										<button
											key={notification.id}
											type="button"
											className={`w-full text-left cursor-pointer rounded-lg border p-3 transition-colors hover:bg-gray-50 dark:hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
												!notification.read
													? "border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20"
													: ""
											}`}
											onClick={() => handleNotificationClick(notification)}
										>
											<div className="flex items-start gap-3">
												{getNotificationIcon(notification.type)}
												<div className="flex-1 space-y-1">
													<p className="text-sm font-medium">
														{notification.title}
													</p>
													<p className="text-xs text-gray-600 dark:text-gray-400">
														{notification.message}
													</p>
													<p className="text-xs text-gray-500">
														{notification.timestamp.toLocaleDateString()} at{" "}
														{notification.timestamp.toLocaleTimeString()}
													</p>
												</div>
											</div>
										</button>
									))
								)}
							</div>
						</SheetContent>
					</Sheet>

					{/* User Menu */}
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button variant="ghost" className="relative h-8 w-8 rounded-full">
								<Avatar className="h-8 w-8">
									<AvatarImage src={user.avatar} alt={user.name} />
									<AvatarFallback>
										{user.name
											.split(" ")
											.map((n) => n[0])
											.join("")}
									</AvatarFallback>
								</Avatar>
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent className="w-56" align="end" forceMount>
							<DropdownMenuLabel className="font-normal">
								<div className="flex flex-col space-y-1">
									<p className="text-sm font-medium leading-none">
										{user.name}
									</p>
									<p className="text-xs leading-none text-gray-600 dark:text-gray-400">
										{user.email}
									</p>
								</div>
							</DropdownMenuLabel>
							<DropdownMenuSeparator />
							<DropdownMenuItem>
								<User className="mr-2 h-4 w-4" />
								<span>Profile</span>
							</DropdownMenuItem>
							<DropdownMenuItem>
								<Settings className="mr-2 h-4 w-4" />
								<span>Settings</span>
							</DropdownMenuItem>
							<DropdownMenuSeparator />
							<DropdownMenuItem>
								<LogOut className="mr-2 h-4 w-4" />
								<span>Log out</span>
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>
			</div>

			{/* Mobile Search Input */}
			{isMobileSearchOpen && (
				<div className="border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 px-6 py-3 md:hidden">
					<div className="relative">
						<Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400 z-10 pointer-events-none" />
						<Input
							placeholder="Search transactions, accounts..."
							value={searchQuery}
							onChange={handleSearchChange}
							className="w-full pl-10"
							autoFocus
						/>
					</div>
				</div>
			)}
		</header>
	);
}
