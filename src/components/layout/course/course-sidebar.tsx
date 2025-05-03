import { Link } from "wouter";
import { SidebarIcon, SidebarIconStyle } from "../sidebar-icon";
import { SidebarItem } from "./sidebar-item";
import type { Course } from "../../../bindings";

export interface CourseSidebarItem {
	id: number;
	name: string;
}

export interface CourseSidebarProps {
	course: Course;
	items?: CourseSidebarItem[];
}

export const CourseSidebar = ({ items, course }: CourseSidebarProps) => {
	return (
		<div className="flex bg-wood-700 border-r border-ivory/10 min-h-svh select-none flex-col space-y-4">
			<div className="border-b w-full border-ivory/10 min-h-12 content-center">
				<div className="p-2 flex">
					<span className="font-bold flex flex-row gap-3 items-center">
						<SidebarIcon iconStyle={SidebarIconStyle.GOO}>{course.name[0]}</SidebarIcon>
						{course.name}
					</span>
				</div>
			</div>
			<div className="flex flex-col space-y-2 p-2">
				{items?.map((item) => (
					// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
					<Link href={`/course/${course.id}/${item.id}`}>
						<SidebarItem>{item.name}</SidebarItem>
					</Link>
				))}
			</div>
		</div>
	);
};
