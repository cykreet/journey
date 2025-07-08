import clsx from "clsx";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";
import React from "react";

export const SidebarItem = ({
	icon,
	active,
	children,
	loading,
}: {
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	active?: boolean;
	loading?: boolean;
	children?: ReactNode;
}) => {
	const Icon = icon;
	const classNames = clsx(
		"w-full hover:bg-ivory/8 py-1 flex px-3 flex-inline text-wood-100 text-sm items-center gap-2 select-none rounded-md cursor-pointer text-ellipsis overflow-hidden whitespace-nowrap",
		active && "bg-ivory/10 text-tan!",
		loading && "animate-pulse h-6 bg-ivory/10",
	);

	return (
		<div className={classNames} style={{ width: loading ? `${Math.floor(Math.random() * 100)}%` : "auto" }}>
			{!loading && (
				<React.Fragment>
					{Icon && <Icon />}
					{children}
				</React.Fragment>
			)}
		</div>
	);
};
