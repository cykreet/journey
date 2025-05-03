import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { CourseSidebar, type CourseSidebarItem } from "./course-sidebar";
import type { Course } from "../../../bindings";

export interface CourseLayoutProps {
	course: Course;
	sidebarItems?: CourseSidebarItem[];
	children?: React.ReactNode;
}

export const CourseLayout = ({ children, course, sidebarItems }: CourseLayoutProps) => {
	return (
		<PanelGroup className="flex flex-row w-full max-h-svh space-x-20" direction={"horizontal"} autoSaveId="course">
			<Panel className="m-0" defaultSize={20} minSize={10} maxSize={25}>
				<CourseSidebar course={course} items={sidebarItems} />
			</Panel>
			<PanelResizeHandle />
			<Panel className="container mx-auto h-full">
				<div className="flex h-full flex-row overflow-y-auto">
					<div className="flex flex-col space-y-8 mt-10 w-full">
						<div className="pb-30">{children}</div>
					</div>
					<div className="min-w-62 p-2 mx-10 bg-ivory/20 rounded-md h-fit min-h-20 sticky mt-30 top-24">toc</div>
				</div>
			</Panel>
		</PanelGroup>
	);
};
