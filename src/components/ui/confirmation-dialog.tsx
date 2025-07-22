/**
 * Confirmation Dialog Component
 * A reusable confirmation dialog that follows the application's design system
 */

import { AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";

interface ConfirmationDialogProps {
	/** Whether the dialog is open */
	open: boolean;
	/** Callback to handle dialog open/close state */
	onOpenChange: (open: boolean) => void;
	/** Title of the confirmation dialog */
	title?: string;
	/** Message to display in the dialog */
	message: string;
	/** Text for the confirm button */
	confirmText?: string;
	/** Text for the cancel button */
	cancelText?: string;
	/** Variant for the confirm button */
	confirmVariant?:
		| "default"
		| "destructive"
		| "outline"
		| "secondary"
		| "ghost"
		| "link";
	/** Callback when user confirms */
	onConfirm: () => void;
	/** Callback when user cancels (optional, defaults to closing dialog) */
	onCancel?: () => void;
	/** Whether to show an icon in the dialog */
	showIcon?: boolean;
	/** Whether the confirm action is loading */
	isLoading?: boolean;
}

export function ConfirmationDialog({
	open,
	onOpenChange,
	title = "Confirm Action",
	message,
	confirmText = "Confirm",
	cancelText = "Cancel",
	confirmVariant = "default",
	onConfirm,
	onCancel,
	showIcon = true,
	isLoading = false,
}: ConfirmationDialogProps) {
	const handleConfirm = () => {
		onConfirm();
		// Don't automatically close the dialog - let the parent handle it
		// This allows for loading states and error handling
	};

	const handleCancel = () => {
		if (onCancel) {
			onCancel();
		} else {
			onOpenChange(false);
		}
	};

	const handleOpenChange = (newOpen: boolean) => {
		// Only allow closing if not loading
		if (!newOpen && isLoading) {
			return;
		}
		onOpenChange(newOpen);
	};

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent className="sm:max-w-md">
				<DialogHeader>
					<DialogTitle className="flex items-center gap-2">
						{showIcon && confirmVariant === "destructive" && (
							<AlertTriangle className="h-5 w-5 text-destructive" />
						)}
						{title}
					</DialogTitle>
					<DialogDescription className="text-left">{message}</DialogDescription>
				</DialogHeader>
				<DialogFooter className="flex-col-reverse sm:flex-row sm:justify-end gap-2">
					<Button
						variant="outline"
						onClick={handleCancel}
						disabled={isLoading}
						className="w-full sm:w-auto"
					>
						{cancelText}
					</Button>
					<Button
						variant={confirmVariant}
						onClick={handleConfirm}
						disabled={isLoading}
						className="w-full sm:w-auto"
					>
						{isLoading ? "Processing..." : confirmText}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
