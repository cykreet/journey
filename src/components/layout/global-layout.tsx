import type React from "react";
import { useRoute } from "wouter";
import IconSpeaker from "~icons/tabler/device-speaker-filled";
import { useVersion } from "../../hooks/version";
import { MenuLayout } from "./menu/menu-layout";
import type { MenuSidebarSectionProps } from "./menu/menu-sidebar";
import { Sidebar } from "./sidebar";

export function GlobalLayout({ children }: { children: React.ReactNode }) {
	const [isCourseRoute] = useRoute("/course/:id?/:page?");
	const version = useVersion();

	const versionTag = <span className="text-xs text-steel-100/50 font-normal ml-1">v{version}</span>;
	const homeMenuItems: MenuSidebarSectionProps[] = [
		{
			subItems: [
				{
					// id: 1,
					icon: IconSpeaker,
					name: "Announcements",
					href: "/announcements",
				},
			],
		},
	];

	return (
		<>
			<Sidebar />
			{/* react-resizable-panels adds children of PanelGroup to a an empty div, which breaks widths, so we have to target it with *:w-full */}
			<div className="flex w-full border-t border-border border-l rounded-tl-md h-full *:w-full">
				{(isCourseRoute == false && (
					<MenuLayout key="home" header={<span>Journey {versionTag}</span>} sidebarSections={homeMenuItems}>
						<div>{children}</div>
					</MenuLayout>
					// course routes have their own menu layout setup
				)) || <div>{children}</div>}
			</div>
		</>
	);
}
