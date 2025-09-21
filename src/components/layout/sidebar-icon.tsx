import clsx from "clsx";
import type { CSSProperties } from "react";

export function SidebarIcon({
	title,
	children,
	className,
	style,
	icon,
	onClick,
}: {
	children: React.ReactNode;
	title?: string;
	className?: string;
	style?: CSSProperties;
	icon?: React.ReactNode;
	onClick?: (event: React.MouseEvent) => void;
}) {
	const classes = clsx("select-none cursor-pointer rounded-md w-8 h-8 font-bold text-center content-center", className);

	return (
		// biome-ignore lint/a11y/useKeyWithClickEvents: <explanation>
		<div title={title} className={classes} style={style} onClick={onClick}>
			{icon || children}
		</div>
	);
}
