import { Link, useLocation } from "wouter";
import { SidebarItem } from "./sidebar-item";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";
import IconInfo from "~icons/tabler/info-circle-filled";
import { Skeleton } from "../../skeleton";

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
	sidebarNotice?: string;
}

export function MenuSidebar({ sections, loading, header, sidebarNotice: notice }: MenuSidebarProps) {
	const [location] = useLocation();

	return (
		<aside className="flex bg-wood-700 border-r border-ivory/10 select-none flex-col h-full min-w-full overflow-hidden">
			<div className="border-b w-full border-ivory/10 min-h-12 content-center">
				<div className="p-2 font-bold">{header}</div>
			</div>
			<div className="flex flex-col space-y-3 p-2 overflow-y-scroll h-full min-w-full">
				{(loading && (
					<div className="flex flex-col space-y-2">
						{[...Array(10)].map((_, i) => (
							// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
							<Skeleton className="h-6  rounded-md" style={{ width: `${Math.min(Math.random() * 130, 100)}%` }} />
						))}
					</div>
				)) || (
					<>
						{notice && (
							<span className="text-xs text-wood-100/50 flex mb-4">
								<IconInfo className="inline-block mr-1" />
								{notice}
							</span>
						)}
						{sections?.map((section) => {
							return (
								<div key={section.id} className="flex flex-col space-y-1">
									<span className="text-xs text-ivory/60">{section.name}</span>
									{section.subItems.map((item) => {
										return (
											<Link href={item.href} key={item.name}>
												<SidebarItem icon={item.icon} active={location === item.href}>
													{item.name}
												</SidebarItem>
											</Link>
										);
									})}
								</div>
							);
						})}
					</>
				)}
			</div>
		</aside>
	);
}
