import { IconArchiveFilled, IconCookieFilled, IconUserFilled } from "@tabler/icons-react";
import { Link } from "wouter";
import { commands } from "../../bindings";
import { useUserCourses } from "../../hooks/useUserCourses";
import { SidebarIcon, SidebarIconStyle } from "./sidebar-icon";

export const Sidebar = () => {
	const courses = useUserCourses();

	return (
		<div className="flex flex-col px-2 py-2 bg-wood-700 h-svh border-r border-ivory/10">
			<div className="flex flex-col space-y-4 overflow-y-auto h-full">
				<Link href="/">
					<SidebarIcon iconStyle={SidebarIconStyle.GOO}>
						<IconCookieFilled className="mx-auto" />
					</SidebarIcon>
				</Link>
				<Link href="/announcements">
					<SidebarIcon iconStyle={SidebarIconStyle.WOOD}>
						<IconArchiveFilled className="mx-auto" />
					</SidebarIcon>
				</Link>
				<hr className="border-ivory/10" />
				{courses?.map((course) => (
					// biome-ignore lint/correctness/useJsxKeyInIterable: <explanation>
					<Link href={`/course/${course.id}`}>
						<SidebarIcon iconStyle={SidebarIconStyle.GOO} style={{ backgroundColor: course.colour ?? "" }}>
							{course.name[0]}
						</SidebarIcon>
					</Link>
				))}
			</div>
			<div className="mt-auto">
				<SidebarIcon iconStyle={SidebarIconStyle.GOO} onClick={() => commands.openLoginWindow("https://google.com")}>
					<IconUserFilled className="mx-auto" />
				</SidebarIcon>
			</div>
		</div>
	);
};
