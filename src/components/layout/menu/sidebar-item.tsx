import clsx from "clsx";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";

export function MenuSidebarItem({
	icon,
	active,
	children,
}: {
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	active?: boolean;
	disabled?: boolean;
	children?: ReactNode;
}) {
	const Icon = icon;
	const classNames = clsx(
		"w-full hover:border-accent hover:bg-accent/10 border-1 border-transparent py-1.5 flex px-3 flex-inline text-sm items-center gap-2 select-none rounded-md cursor-pointer",
		active && "bg-accent/10 text-accent!",
	);

	return (
		<div className={classNames}>
			{Icon && <Icon />}
			<span className="text-ellipsis overflow-hidden whitespace-nowrap">{children}</span>
		</div>
	);
}
