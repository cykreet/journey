import { Link } from "wouter";
import IconJourney from "~icons/journey/journey";
import IconUserFilled from "~icons/tabler/user-filled";
import { commands } from "../../bindings";
import { useCommand } from "../../hooks/useUserCourses";
import { SidebarIcon, SidebarIconStyle } from "./sidebar-icon";

export const Sidebar = ({ onUserClick }: { onUserClick: () => void }) => {
	const courses = useCommand(commands.getUserCourses);

	return (
		<div className="flex flex-col px-3 py-2 bg-wood-700 h-svh border-r border-ivory/10">
			<div className="flex flex-col space-y-3 overflow-y-auto h-full hide-scroll">
				<Link href="/" className="mb-0">
					<IconJourney className="mx-auto w-7 h-7" />
				</Link>
				<hr className="border-ivory/10 my-3" />
				{courses.data?.map((course) => (
					<Link href={`/course/${course.id}`} key={course.id}>
						<SidebarIcon iconStyle={SidebarIconStyle.WOOD} style={{ backgroundColor: course.colour ?? "" }}>
							{course.name[0]}
						</SidebarIcon>
					</Link>
				))}
			</div>
			<div className="mt-auto pt-2">
				<SidebarIcon iconStyle={SidebarIconStyle.GOO} onClick={() => onUserClick()}>
					<IconUserFilled className="mx-auto" />
				</SidebarIcon>
			</div>
		</div>
	);
};
