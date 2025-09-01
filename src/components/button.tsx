import clsx from "clsx";
import { Loading } from "./loading";

export enum ButtonStyle {
	PRIMARY = "bg-goo text-wood hover:bg-goo/80",
	IVORY = "bg-ivory/10 border border-ivory/20 hover:bg-ivory/5",
	BORDERLESS = "bg-ivory/10 hover:bg-ivory/5",
	GHOST = "bg-transparent text-wood hover:bg-ivory/5",
}

export function Button({
	buttonStyle = ButtonStyle.PRIMARY,
	onClick,
	disabled,
	loading,
	className,
	children,
}: {
	buttonStyle?: ButtonStyle;
	onClick: () => void;
	disabled?: boolean;
	loading?: boolean;
	className?: string;
	children: React.ReactNode;
}) {
	const classes = clsx(
		"rounded-md px-1 py-1",
		buttonStyle,
		!disabled && "cursor-pointer",
		disabled && "cursor-not-allowed bg-ivory/10 border border-ivory/10 hover:bg-ivory/10 text-ivory/20!",
		className,
	);

	return (
		<button type="button" className={classes} onClick={() => !disabled && onClick()}>
			{(loading && <Loading />) || children}
		</button>
	);
}
