import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useContext, useEffect, useState } from "react";
import IconAlertSquare from "~icons/tabler/alert-square-filled";
import IconLayoutSidebar from "~icons/tabler/layout-sidebar-filled";
import IconMinus from "~icons/tabler/minus";
import IconSquare from "~icons/tabler/square";
import User from "~icons/tabler/user-filled";
import IconX from "~icons/tabler/x";
import type { SyncError } from "../../bindings";
import { events } from "../../bindings";
import { useLoginWindow } from "../../hooks/login-window";
import { useUser } from "../../hooks/user";
import { Button, ButtonStyle } from "../button";
import Dropdown from "../dropdown";
import { ModuleContext } from "./module-context";
import { SidebarContext } from "./sidebar-context";
import { LoginContext } from "./login-context";
import { AuthStatus } from "../../types";

export function WindowControls({ children }: { children: React.ReactNode }) {
	const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
	const [moduleName, setModuleName] = useState<string | undefined>(undefined);
	const [syncError, setSyncError] = useState<SyncError | undefined>(undefined);
	const [moduleLoading, setModuleLoading] = useState(false);
	const loginContext = useContext(LoginContext);
	const { openLoginWindow, loading: loginLoading } = useLoginWindow();
	const { userName, host } = useUser();

	const statusColour = moduleLoading ? "bg-steel-100" : syncError ? "bg-crimson" : "bg-accent";
	const shouldReauthenticate =
		host && loginContext?.authStatus !== AuthStatus.Success && syncError?.code === "invalidtoken";

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		const unlistenPromise = events.syncErrorEvent.listen((event) => {
			setSyncError(event.payload);
		});

		return () => {
			setSyncError(undefined);
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [moduleName]);

	return (
		<div className="w-full h-full flex flex-col">
			<div data-tauri-drag-region className="bg-steel-700 grid grid-cols-3 w-full">
				<div data-tauri-drag-region className="py-1 ml-2">
					<Button onClick={() => setSidebarCollapsed(!sidebarCollapsed)} buttonStyle={ButtonStyle.GHOST}>
						<IconLayoutSidebar className="w-5 h-5" />
					</Button>
				</div>
				<div data-tauri-drag-region className="flex justify-center my-auto h-8">
					{moduleName && (
						// todo: display module error state on click with a popout or something
						<Button
							onClick={() => {}}
							buttonStyle={ButtonStyle.BORDERLESS}
							className="inline my-auto text-xs px-3 w-fit"
						>
							<div
								className={`w-1.5 h-1.5 mr-2 rounded-full inline-block ${statusColour} ${moduleLoading && "animate-pulse"}`}
							/>
							{moduleName}
						</Button>
					)}
				</div>
				<div data-tauri-drag-region className="flex flex-row items-center h-full justify-end">
					{host && (
						<Dropdown>
							<Dropdown.Trigger>
								<Button buttonStyle={ButtonStyle.BORDERLESS} className="relative rounded-md mr-2 flex items-center">
									{shouldReauthenticate && (
										<div className="rounded-full animate-bounce h-1.5 w-1.5 absolute top-0 right-0 bg-crimson" />
									)}
									<User className="w-4 h-4" />
								</Button>
							</Dropdown.Trigger>
							<Dropdown.Menu>
								<Dropdown.ItemContainer>
									<div className="flex flex-col justify-center *:text-ellipsis *:overflow-hidden">
										{userName && <span>{userName}</span>}
										<span title={host} className="text-xs opacity-50">
											{new URL(host).hostname}
										</span>
									</div>
								</Dropdown.ItemContainer>
								{shouldReauthenticate && (
									<>
										<Dropdown.Divider />
										<Dropdown.ItemContainer>
											<Dropdown.Item
												loading={loginLoading}
												onClick={() => openLoginWindow(host)}
												icon={IconAlertSquare}
												buttonStyle={ButtonStyle.CRIMSON}
												title="Your session has expired, you'll need to reauthenticate with Moodle"
											>
												Reauthenticate
											</Dropdown.Item>
										</Dropdown.ItemContainer>
									</>
								)}
							</Dropdown.Menu>
						</Dropdown>
					)}
					<Button
						onClick={() => getCurrentWebviewWindow().minimize()}
						buttonStyle={ButtonStyle.GHOST}
						className="text-steel-100 h-full rounded-none"
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
						className="text-steel-100 h-full rounded-none"
					>
						<IconSquare className="w-3.5 h-3.5 mx-2" />
					</Button>
					<Button
						onClick={() => getCurrentWebviewWindow().close()}
						buttonStyle={ButtonStyle.GHOST}
						className="text-steel-100 h-full hover:bg-crimson rounded-none"
					>
						<IconX className="w-4 h-4 mx-2" />
					</Button>
				</div>
			</div>
			<div className="flex flex-row max-h-full overflow-hidden w-full h-full">
				<SidebarContext.Provider
					value={{
						collapsed: sidebarCollapsed,
						setCollapsed: setSidebarCollapsed,
					}}
				>
					<ModuleContext.Provider
						value={{
							name: moduleName,
							loading: moduleLoading,
							error: syncError,
							setName: setModuleName,
							setError: setSyncError,
							setLoading: setModuleLoading,
						}}
					>
						{children}
					</ModuleContext.Provider>
				</SidebarContext.Provider>
			</div>
		</div>
	);
}
