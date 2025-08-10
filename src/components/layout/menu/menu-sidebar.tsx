import { Link, useLocation } from "wouter";
import { SidebarItem } from "./sidebar-item";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";
import React from "react";

export interface MenuSidebarSection {
	id?: number;
	name?: string;
	subItems: MenuSidebarItem[];
}

export interface MenuSidebarItem {
	name: string;
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
	href?: string;
}

export interface MenuSidebarProps {
	header: ReactNode;
	loading?: boolean;
	sections?: MenuSidebarSection[];
}

export const MenuSidebar = ({ sections, loading, header }: MenuSidebarProps) => {
	const [location] = useLocation();

	return (
		<div className="flex bg-wood-700 border-r border-ivory/10 min-h-svh select-none flex-col space-y-4 rounded-r-2xl h-full">
			<div className="border-b w-full border-ivory/10 min-h-12 content-center">
				<div className="p-2 font-bold">{header}</div>
			</div>
			<div className="flex flex-col space-y-3 p-2 overflow-y-auto h-full">
				{(loading && (
					<div className="flex flex-col space-y-2">
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
						<SidebarItem loading />
					</div>
				)) ||
					sections?.map((section) => {
						return (
							// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
							<React.Fragment>
								<span className="text-xs text-ivory/60">{section.name}</span>
								{section.subItems?.map((item) => {
									return (
										// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
										<div className="flex flex-col space-y-1">
											<Link href={item.href ?? "#"} key={item.name}>
												<SidebarItem icon={item.icon} active={location === item.href}>
													{item.name}
												</SidebarItem>
											</Link>
										</div>
									);
								})}
							</React.Fragment>
						);
					})}
			</div>
		</div>
	);
};
