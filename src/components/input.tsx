import clsx from "clsx";
import type { HTMLInputTypeAttribute } from "react";

export function Input({
	type = "text",
	label,
	disabled,
	placeholder,
	className,
	value,
	onChange,
	onEnter,
}: {
	label?: string;
	placeholder?: string;
	className?: string;
	type?: HTMLInputTypeAttribute;
	value?: string;
	disabled?: boolean;
	onChange?: (value: string) => void;
	onEnter?: () => void;
}) {
	const containerClasses = clsx("flex flex-col space-y-1", className);
	const classNames = clsx(
		"border border-border rounded-md p-1.5 bg-steel-700 text-steel-100 focus-within:border-accent",
		{
			"cursor-not-allowed opacity-50": disabled,
		},
	);

	return (
		<div className={containerClasses}>
			<span className="text-sm">{label}</span>
			<div className={classNames}>
				<input
					disabled={disabled}
					type={type}
					className="w-full focus:outline-none group"
					placeholder={placeholder}
					value={value}
					onChange={(event) => onChange?.(event.target.value)}
					onKeyDown={(event) => event.key === "Enter" && onEnter?.()}
				/>
			</div>
		</div>
	);
}
