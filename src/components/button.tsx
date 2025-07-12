import clsx from "clsx";
import { Loading } from "./loading";

export enum ButtonStyle {
	PRIMARY = "bg-goo text-wood hover:bg-goo/80",
	IVORY = "bg-ivory/10 border border-ivory/20 hover:bg-ivory/5",
}

export const Button = ({
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
}) => {
	const classes = clsx(
		"rounded-md px-5 py-1",
		buttonStyle,
		!disabled && "cursor-pointer",
		disabled && "cursor-not-allowed bg-ivory/10 border border-ivory/10 hover:bg-ivory/10 text-ivory/20!",
		className,
	);

	return (
		// biome-ignore lint/a11y/useButtonType: <explanation>
		// biome-ignore lint/a11y/useButtonType: <explanation>
		<button className={classes} onClick={() => !disabled && onClick}>
			{(loading && <Loading />) || children}
		</button>
	);
};
