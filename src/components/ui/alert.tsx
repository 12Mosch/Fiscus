import { cva, type VariantProps } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@/lib/utils";

const alertVariants = cva(
	"relative w-full rounded-lg border px-4 py-3 text-sm [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground [&>svg~*]:pl-7",
	{
		variants: {
			variant: {
				default: "bg-background text-foreground",
				destructive:
					"border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive",
				warning:
					"border-yellow-500/50 text-yellow-800 bg-yellow-50 dark:border-yellow-500 dark:text-yellow-200 dark:bg-yellow-900/20 [&>svg]:text-yellow-600 dark:[&>svg]:text-yellow-400",
				success:
					"border-green-500/50 text-green-800 bg-green-50 dark:border-green-500 dark:text-green-200 dark:bg-green-900/20 [&>svg]:text-green-600 dark:[&>svg]:text-green-400",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	},
);

function Alert({
	className,
	variant,
	...props
}: React.ComponentProps<"div"> & VariantProps<typeof alertVariants>) {
	return (
		<div
			role="alert"
			className={cn(alertVariants({ variant }), className)}
			{...props}
		/>
	);
}

function AlertTitle({ className, ...props }: React.ComponentProps<"h5">) {
	return (
		<h5
			className={cn("mb-1 font-medium leading-none tracking-tight", className)}
			{...props}
		/>
	);
}

function AlertDescription({
	className,
	...props
}: React.ComponentProps<"div">) {
	return (
		<div
			className={cn("text-sm [&_p]:leading-relaxed", className)}
			{...props}
		/>
	);
}

export { Alert, AlertTitle, AlertDescription };
