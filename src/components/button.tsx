import clsx from "clsx";

export enum ButtonStyle {
	PRIMARY = "bg-goo text-wood hover:bg-goo/80",
	IVORY = "bg-ivory/10 border border-ivory/20 hover:bg-ivory/5",
}

export const Button = ({
	buttonStyle = ButtonStyle.PRIMARY,
	onClick,
	children,
}: { buttonStyle?: ButtonStyle; onClick: () => void; children: React.ReactNode }) => {
	const classes = clsx("rounded-md px-5 py-1 cursor-pointer", buttonStyle);

	return (
		// biome-ignore lint/a11y/useButtonType: <explanation>
		// biome-ignore lint/a11y/useButtonType: <explanation>
		<button className={classes} onClick={onClick}>
			{children}
		</button>
	);
};
