import { useRoute } from "wouter";
import { commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarItem } from "../components/layout/menu/menu-sidebar";
import { SidebarIcon, SidebarIconStyle } from "../components/layout/sidebar-icon";
import { useCommand } from "../hooks/useUserCourses";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data, error } = useCommand(commands.getUserCourses);
	const courseId = params?.id;
	const pageId = params?.page;

	const course = data?.find((course) => course.id === Number(courseId));
	if (!match || error || !course) return <div>{error}</div>;
	const coursePages: MenuSidebarItem[] = [
		{
			id: 1,
			name: "Introduction",
			href: `/course/${courseId}/introduction`,
		},
	];

	return (
		<MenuLayout
			header={
				<span className="font-bold flex flex-row gap-3 items-center">
					<SidebarIcon iconStyle={SidebarIconStyle.GOO}>{course.name[0]}</SidebarIcon>
					{course.name}
				</span>
			}
			sidebarItems={coursePages}
		>
			<h2>Introduction</h2>
			<span>
				course id: {courseId}, page name: {pageId}
			</span>
		</MenuLayout>
	);
};
