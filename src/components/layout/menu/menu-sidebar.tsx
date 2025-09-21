import { Link, useLocation } from "wouter";
import { MenuSidebarItem } from "./sidebar-item";
import type { ForwardRefExoticComponent, ReactNode, SVGProps } from "react";
import IconInfo from "~icons/tabler/info-circle-filled";
import { Skeleton } from "../../skeleton";

export interface MenuSidebarSectionProps {
	id?: number;
	name?: string;
	subItems: MenuSidebarItemProps[];
}

export interface MenuSidebarItemProps {
	name: string;
	href: string;
	icon?: ForwardRefExoticComponent<SVGProps<SVGSVGElement>>;
}

export interface MenuSidebarProps {
	header: ReactNode;
	loading?: boolean;
	sections?: MenuSidebarSectionProps[];
	sidebarNotice?: string;
}

export function MenuSidebar({ sections, loading, header, sidebarNotice: notice }: MenuSidebarProps) {
	const [location] = useLocation();

	return (
		<aside className="flex bg-steel-700 border-r border-border select-none flex-col h-full min-w-full overflow-hidden">
			<div className="border-b w-full border-border min-h-12 content-center flex items-center">
				<div className="p-2 font-bold">{header}</div>
			</div>
			<div className="flex flex-col space-y-3 p-2 overflow-y-scroll h-full min-w-full">
				{(loading && (
					<div className="flex flex-col space-y-2">
						{[...Array(10)].map((_) => (
							// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
							<Skeleton className="h-6  rounded-md" style={{ width: `${Math.min(Math.random() * 130, 100)}%` }} />
						))}
					</div>
				)) || (
					<>
						{notice && (
							<span className="text-xs text-steel-300 flex mb-4">
								<IconInfo className="inline-block mr-1" />
								{notice}
							</span>
						)}
						{sections?.map((section) => {
							return (
								<div key={section.id} className="flex flex-col space-y-1">
									<span className="text-xs text-steel-200">{section.name}</span>
									{section.subItems.map((item) => {
										return (
											<Link href={item.href} key={item.name}>
												<MenuSidebarItem icon={item.icon} active={location === item.href}>
													{item.name}
												</MenuSidebarItem>
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
