import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { MenuSidebar, type MenuSidebarSection } from "./menu-sidebar";

export interface MenuLayoutProps {
	key: string;
	header: React.ReactNode;
	sidebarSections?: MenuSidebarSection[];
	loading?: boolean;
	children?: React.ReactNode;
}

export const MenuLayout = ({ key, children, header, loading, sidebarSections }: MenuLayoutProps) => {
	return (
		<PanelGroup key={key} className="flex flex-row w-full h-full" direction={"horizontal"} autoSaveId="sidebar">
			{/* todo: replace with sidebar state */}
			{true && (
				<>
					<Panel className="m-0 h-full w-full" defaultSize={20} minSize={20} maxSize={30} order={1}>
						<MenuSidebar loading={loading} header={header} sections={sidebarSections} />
					</Panel>
					<PanelResizeHandle />
				</>
			)}
			<Panel className="m-0 h-full" order={2}>
				<div className="flex container mx-auto overflow-y-auto h-full w-full">
					<div className="flex flex-col mt-10 w-full mx-20">{children}</div>
				</div>
			</Panel>
		</PanelGroup>
	);
};
