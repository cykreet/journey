import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useState } from "react";
import IconLayoutSidebar from "~icons/tabler/layout-sidebar-filled";
import IconMinus from "~icons/tabler/minus";
import IconSquare from "~icons/tabler/square";
import IconSquares from "~icons/tabler/squares";
import IconX from "~icons/tabler/x";
import { Button, ButtonStyle } from "../button";
import { SidebarContext } from "./sidebar-context";

export const WindowControls = ({ children }: { children: React.ReactNode }) => {
	const [maximised, setMaximised] = useState(false);
	const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

	return (
		<div className="w-full h-full flex flex-col">
			<div data-tauri-drag-region className="bg-wood-700 justify-between flex flex-row">
				<div className="py-1 ml-2">
					<Button
						onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
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
				<SidebarContext.Provider value={sidebarCollapsed}>{children}</SidebarContext.Provider>
			</div>
		</div>
	);
};
