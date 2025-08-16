import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { MenuSidebar, type MenuSidebarSection } from "./menu-sidebar";

export interface MenuLayoutProps {
	key: string;
	header: React.ReactNode;
	sidebarSections?: MenuSidebarSection[];
	loading?: boolean;
	children?: React.ReactNode;
	collapseSidebar?: boolean;
}

export const MenuLayout = ({
	key,
	children,
	header,
	loading,
	sidebarSections,
	collapseSidebar = false,
}: MenuLayoutProps) => {
	return (
		<PanelGroup key={key} className="flex flex-row w-full h-full" direction={"horizontal"} autoSaveId="sidebar">
			{collapseSidebar == false && (
				<>
					<Panel className="m-0 h-full w-full" defaultSize={20} minSize={20} maxSize={30} order={1}>
						<MenuSidebar loading={loading} header={header} sections={sidebarSections} />
					</Panel>
					<PanelResizeHandle />
				</>
			)}
			<Panel className="m-0 h-full" order={2}>
				<div className="container mx-auto overflow-y-auto h-full w-full">
					<div className="flex flex-row h-full w-full">
						<div className="flex flex-col space-y-8 mt-10 w-full mx-20">{children}</div>
					</div>
				</div>
			</Panel>
		</PanelGroup>
	);
};
