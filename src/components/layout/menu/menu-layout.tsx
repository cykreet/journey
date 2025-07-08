import IconAlignJustified from "~icons/tabler/align-justified";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { MenuSidebar, type MenuSidebarSection } from "./menu-sidebar";

export interface MenuLayoutProps {
	header: React.ReactNode;
	sidebarSections?: MenuSidebarSection[];
	loading?: boolean;
	tocItems?: MenuSidebarSection[];
	children?: React.ReactNode;
}

export const MenuLayout = ({ children, header, tocItems, loading, sidebarSections }: MenuLayoutProps) => {
	return (
		<PanelGroup className="flex flex-row w-full max-h-svh space-x-20" direction={"horizontal"} autoSaveId="sidebar">
			<Panel className="m-0" defaultSize={20} minSize={10} maxSize={25}>
				<MenuSidebar loading={loading} header={header} sections={sidebarSections} />
			</Panel>
			<PanelResizeHandle />
			<Panel className="container mx-auto h-full">
				<div className="flex h-full flex-row overflow-y-auto">
					<div className="flex flex-col space-y-8 mt-10 w-full">
						<div className="pb-30">{children}</div>
					</div>
					{tocItems && (
						<div className="min-w-62 p-2 mx-10 rounded-md h-fit min-h-20 sticky mt-30 top-24">
							<span className="flex flex-row gap-2">
								<IconAlignJustified />
								On this page
							</span>
						</div>
					)}
				</div>
			</Panel>
		</PanelGroup>
	);
};
