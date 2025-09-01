import clsx from "clsx";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";

export function SidebarItem({
	icon,
	active,
	children,
	loading,
}: {
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	active?: boolean;
	disabled?: boolean;
	loading?: boolean;
	children?: ReactNode;
}) {
	const Icon = icon;
	const classNames = clsx(
		"w-full hover:border-goo hover:bg-goo/10 border-1 border-transparent py-1.5 flex px-3 flex-inline text-wood-100 text-sm items-center gap-2 select-none rounded-md cursor-pointer",
		active && "bg-goo/10 text-goo!",
		loading && "animate-pulse h-6 bg-ivory/8",
	);

	return (
		<div className={classNames} style={{ width: loading ? `${Math.floor(Math.random() * 100)}%` : "auto" }}>
			{!loading && (
				<>
					{Icon && <Icon />}
					<span className="text-ellipsis overflow-hidden whitespace-nowrap">{children}</span>
				</>
			)}
		</div>
	);
}
