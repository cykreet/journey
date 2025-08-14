import type React from "react";
import { useRoute } from "wouter";
import IconSpeaker from "~icons/tabler/device-speaker-filled";
import { MenuLayout } from "./menu/menu-layout";
import type { MenuSidebarSection } from "./menu/menu-sidebar";
import { Sidebar } from "./sidebar";

export const GlobalLayout = ({ children }: { children: React.ReactNode }) => {
	const [isCourseRoute] = useRoute("/course/:id?/:page?");

	const homeMenuItems: MenuSidebarSection[] = [
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
		<div className="flex flex-row max-h-svh overflow-hidden w-full">
			<Sidebar />
			<div className="w-full">
				{(isCourseRoute == false && (
					<MenuLayout key="home" header={"Journey"} sidebarSections={homeMenuItems}>
						<div>{children}</div>
					</MenuLayout>
				)) || <div>{children}</div>}
			</div>
		</div>
	);
};
