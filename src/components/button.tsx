import clsx from "clsx";
import { Loading } from "./loading";
import type { ForwardRefExoticComponent, SVGProps } from "react";

export enum ButtonStyle {
	PRIMARY = "bg-accent text-steel hover:bg-accent/80",
	// IVORY = "bg-ivory/10 border border-ivory/20 hover:bg-ivory/5",
	BORDERLESS = "bg-steel hover:bg-steel-600",
	GHOST = "bg-transparent hover:bg-steel",
	CRIMSON = "bg-crimson/25 text-crimson hover:bg-crimson/40",
}

export interface ButtonProps {
	buttonStyle?: ButtonStyle;
	onClick?: () => void;
	disabled?: boolean;
	loading?: boolean;
	className?: string;
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	title?: string;
	children: React.ReactNode;
}

export function Button({
	buttonStyle = ButtonStyle.PRIMARY,
	onClick,
	disabled,
	loading,
	className,
	children,
	title,
	icon,
}: ButtonProps) {
	const Icon = icon;
	const classes = clsx(
		"rounded-md px-1 py-1 flex flex-inline gap-2 items-center",
		buttonStyle,
		!(disabled || loading) && "cursor-pointer",
		(disabled || loading) && "cursor-not-allowed bg-steel-700 text-steel-300 hover:bg-steel-700",
		className,
	);

	return (
		<button
			title={title}
			type="button"
			className={classes}
			onClick={() => !disabled && !loading && onClick && onClick()}
		>
			{(loading && <Loading />) || (Icon && <Icon className="w-4 h-4" />)}
			{children && <span className="flex-grow text-ellipsis overflow-hidden">{children}</span>}
		</button>
	);
}
