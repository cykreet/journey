import clsx from "clsx";
import type { HTMLInputTypeAttribute } from "react";

export const Input = ({
	type = "text",
	label,
	disabled,
	placeholder,
	className,
	value,
	onChange,
}: {
	label?: string;
	placeholder?: string;
	className?: string;
	type?: HTMLInputTypeAttribute;
	value?: string;
	disabled?: boolean;
	onChange?: (value: string) => void;
}) => {
	const containerClasses = clsx("flex flex-col space-y-1", className);

	return (
		<div className={containerClasses}>
			<span className="text-sm">{label}</span>
			<div className="border border-ivory/10 rounded-md p-1.5 focus-within:border-goo">
				<input
					disabled={disabled}
					type={type}
					className="w-full focus:outline-none group"
					placeholder={placeholder}
					value={value}
					onChange={(event) => onChange?.(event.target.value)}
				/>
			</div>
		</div>
	);
};
