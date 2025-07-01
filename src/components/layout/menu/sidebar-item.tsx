import clsx from "clsx";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";

export const SidebarItem = ({
	icon,
	active,
	children,
}: { icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>; active?: boolean; children: ReactNode }) => {
	const Icon = icon;
	const classNames = clsx(
		"w-full hover:bg-ivory/8 py-1.5 flex px-3 flex-inline items-center gap-2 select-none rounded-md cursor-pointer text-ellipsis overflow-hidden whitespace-nowrap",
		active && "bg-ivory/10",
	);

	return (
		<div className={classNames}>
			{Icon && <Icon />}
			{children}
		</div>
	);
};
