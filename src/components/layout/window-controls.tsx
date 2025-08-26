import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import IconLayoutSidebar from "~icons/tabler/layout-sidebar-filled";
import IconMinus from "~icons/tabler/minus";
import IconSquare from "~icons/tabler/square";
import IconX from "~icons/tabler/x";
import { Button, ButtonStyle } from "../button";
import { SidebarContext } from "./sidebar-context";
import { ModuleContext } from "./module-context";
import { events } from "../../bindings";

export const WindowControls = ({ children }: { children: React.ReactNode }) => {
	const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
	const [moduleName, setModuleName] = useState<string | undefined>(undefined);
	const [moduleError, setModuleError] = useState<string | undefined>(undefined);
	const [moduleLoading, setModuleLoading] = useState(false);

	const statusColour = moduleLoading ? "bg-wood-100" : moduleError ? "bg-rose-500" : "bg-goo";

	useEffect(() => {
		const unlistenPromise = events.moduleErrorEvent.listen((event) => {
			setModuleError(event.payload);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, []);

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
				{moduleName && (
					// todo: display module error state on click with a popout or something
					<Button onClick={() => {}} buttonStyle={ButtonStyle.BORDERLESS} className="inline my-auto text-xs px-3">
						<div
							className={`w-1.5 h-1.5 mr-2 rounded-full inline-block ${statusColour} ${moduleLoading && "animate-pulse"}`}
						/>
						{moduleName}
					</Button>
				)}
				<div className="flex flex-row items-center *:rounded-none! h-full">
					<Button
						onClick={() => getCurrentWebviewWindow().minimize()}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100 h-full"
					>
						<IconMinus className="w-4 h-4 mx-2" />
					</Button>
					<Button
						onClick={async () => {
							const window = getCurrentWebviewWindow();
							if (await window.isMaximized()) window.unmaximize();
							else window.maximize();
						}}
						buttonStyle={ButtonStyle.GHOST}
						className="text-wood-100 h-full"
					>
						<IconSquare className="w-3.5 h-3.5 mx-2" />
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
				<SidebarContext.Provider value={sidebarCollapsed}>
					<ModuleContext.Provider
						value={{
							name: moduleName,
							loading: moduleLoading,
							setName: setModuleName,
							setLoading: setModuleLoading,
						}}
					>
						{children}
					</ModuleContext.Provider>
				</SidebarContext.Provider>
			</div>
		</div>
	);
};
