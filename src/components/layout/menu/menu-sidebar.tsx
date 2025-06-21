import { Link, useLocation } from "wouter";
import { SidebarItem } from "./sidebar-item";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";

export interface MenuSidebarItem {
	id: number;
	name: string;
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	href?: string;
	subItems?: MenuSidebarItem[];
}

export interface MenuSidebarProps {
	header: ReactNode;
	items?: MenuSidebarItem[];
}

export const MenuSidebar = ({ items, header }: MenuSidebarProps) => {
	const [location] = useLocation();

	return (
		<div className="flex bg-wood-700 border-r border-ivory/10 min-h-svh select-none flex-col space-y-4 rounded-r-2xl">
			<div className="border-b w-full border-ivory/10 min-h-12 content-center">
				<div className="p-2 font-bold">{header}</div>
			</div>
			<div className="flex flex-col space-y-2 p-2">
				{items?.map((item) => (
					<Link href={item.href ?? "#"} key={item.id}>
						<SidebarItem icon={item.icon} active={location === item.href}>
							{item.name}
						</SidebarItem>
					</Link>
				))}
			</div>
		</div>
	);
};
