import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { MenuSidebar, type MenuSidebarSection } from "./menu-sidebar";
import { useContext } from "react";
import { SidebarContext } from "../sidebar-context";

export interface MenuLayoutProps {
	key: string;
	header: React.ReactNode;
	sidebarSections?: MenuSidebarSection[];
	loading?: boolean;
	children?: React.ReactNode;
	sidebarNotice?: string;
}

export const MenuLayout = ({ key, children, header, loading, sidebarSections, sidebarNotice }: MenuLayoutProps) => {
	const sidebarCollapsed = useContext(SidebarContext);

	return (
		<PanelGroup key={key} className="flex flex-row w-full h-full" direction={"horizontal"} autoSaveId="sidebar">
			{sidebarCollapsed == false && (
				<>
					<Panel className="m-0 h-full w-full" defaultSize={20} minSize={20} maxSize={30} order={1}>
						<MenuSidebar loading={loading} header={header} sections={sidebarSections} sidebarNotice={sidebarNotice} />
					</Panel>
					<PanelResizeHandle className="cursor-ew-resize" />
				</>
			)}
			<Panel className="m-0 h-full flex-1" order={2}>
				{/* overflow-y-auto here preserves scroll position, as opposed to being on a child element */}
				<div className="flex flex-col h-full w-full overflow-y-auto">
					<div className="flex flex-col w-full">{children}</div>
				</div>
			</Panel>
		</PanelGroup>
	);
};
