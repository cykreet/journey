import clsx from "clsx";
import type { CSSProperties } from "react";

export const SidebarIconStyle = {
	GOO: "bg-goo text-wood",
	WOOD: "bg-wood-300 text-wood",
};

export function SidebarIcon({
	title,
	children,
	className,
	style,
	icon,
	onClick,
	iconStyle,
}: {
	children: React.ReactNode;
	title?: string;
	className?: string;
	style?: CSSProperties;
	icon?: React.ReactNode;
	onClick?: (event: React.MouseEvent) => void;
	iconStyle?: (typeof SidebarIconStyle)[keyof typeof SidebarIconStyle];
}) {
	const classes = clsx(
		"select-none cursor-pointer rounded-md w-8 h-8 font-bold text-center content-center",
		className,
		iconStyle,
	);

	return (
		// biome-ignore lint/a11y/useKeyWithClickEvents: <explanation>
		<div title={title} className={classes} style={style} onClick={onClick}>
			{icon || children}
		</div>
	);
}
