import { Link, useLocation } from "wouter";
import { SidebarItem } from "./sidebar-item";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";

export interface MenuSidebarSection {
	id?: number;
	name?: string;
	subItems: MenuSidebarItem[];
}

export interface MenuSidebarItem {
	name: string;
	href: string;
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
}

export interface MenuSidebarProps {
	header: ReactNode;
	loading?: boolean;
	sections?: MenuSidebarSection[];
}

export const MenuSidebar = ({ sections, loading, header }: MenuSidebarProps) => {
	const [location] = useLocation();

	return (
		<aside className="flex bg-wood-700 border-r border-ivory/10 select-none flex-col h-full min-w-full overflow-hidden">
			<div className="border-b w-full border-ivory/10 min-h-12 content-center">
				<div className="p-2 font-bold">{header}</div>
			</div>
			<div className="flex flex-col space-y-3 p-2 overflow-y-scroll h-full min-w-full">
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
							<>
								{/* // biome-ignore lint/correctness/useJsxKeyInIterable: <explanation> */}
								<span className="text-xs text-ivory/60">{section.name}</span>
								<div className="flex flex-col space-y-1">
									{section.subItems?.map((item) => {
										return (
											<Link href={item.href} key={item.name}>
												<SidebarItem icon={item.icon} active={location === item.href}>
													{item.name}
												</SidebarItem>
											</Link>
										);
									})}
								</div>
							</>
						);
					})}
			</div>
		</aside>
	);
};
