import type React from "react";
import { useRoute } from "wouter";
import IconSpeaker from "~icons/tabler/device-speaker-filled";
import IconLayoutSidebar from "~icons/tabler/layout-sidebar-filled";
import IconX from "~icons/tabler/x";
import IconMinus from "~icons/tabler/minus";
import IconSquare from "~icons/tabler/square";
import IconSquares from "~icons/tabler/squares";
import { Button, ButtonStyle } from "../button";
import { MenuLayout } from "./menu/menu-layout";
import type { MenuSidebarSection } from "./menu/menu-sidebar";
import { Sidebar } from "./sidebar";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useState } from "react";

export const GlobalLayout = ({ children }: { children: React.ReactNode }) => {
	const [isCourseRoute] = useRoute("/course/:id?/:page?");
	const [maximised, setMaximised] = useState(false);
	const [collapseSidebar, setCollapseSidebar] = useState(false);

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
		<div className="w-full h-full flex flex-col">
			<div data-tauri-drag-region className="bg-wood-700 justify-between flex flex-row">
				<div className="py-1 ml-2">
					<Button
						onClick={() => setCollapseSidebar(!collapseSidebar)}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100"
					>
						<IconLayoutSidebar className="w-5 h-5" />
					</Button>
				</div>
				<div className="flex flex-row items-center *:rounded-none! h-full">
					<Button
						onClick={() => getCurrentWebviewWindow().minimize()}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100 h-full"
					>
						<IconMinus className="w-4 h-4 mx-2" />
					</Button>
					<Button
						onClick={() => {
							const window = getCurrentWebviewWindow();
							if (maximised) {
								window.unmaximize();
								setMaximised(false);
							} else {
								window.maximize();
								setMaximised(true);
							}
						}}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100 h-full"
					>
						{(maximised && <IconSquares className="w-3.5 h-3.5 mx-2" />) || <IconSquare className="w-3.5 h-3.5 mx-2" />}
					</Button>
					<Button
						onClick={() => getCurrentWebviewWindow().close()}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100 h-full hover:bg-rose-500/40"
					>
						<IconX className="w-4 h-4 mx-2" />
					</Button>
				</div>
			</div>
			<div className="flex flex-row max-h-full overflow-hidden w-full h-full">
				<Sidebar />
				{/* react-resizable-panels adds children of PanelGroup to a an empty div, which breaks widths, so we have to target it with *:w-full */}
				<div className="flex w-full border-t border-ivory/10 border-l rounded-tl-md h-full *:w-full">
					{(isCourseRoute == false && (
						<MenuLayout key="home" header={"Journey"} sidebarSections={homeMenuItems} collapseSidebar={collapseSidebar}>
							<div>{children}</div>
						</MenuLayout>
						// course routes have their own menu layout setup
					)) || <div>{children}</div>}
				</div>
			</div>
		</div>
	);
};
