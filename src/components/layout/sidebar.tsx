import { Link, useLocation } from "wouter";
import IconJourney from "~icons/journey/journey";
// import IconCog from "~icons/tabler/settings-filled";
import { commands } from "../../bindings";
import { useCommand } from "../../hooks/useCommand";
import { SidebarIcon, SidebarIconStyle } from "./sidebar-icon";

export const Sidebar = () => {
	const [location] = useLocation();
	const courses = useCommand(commands.getUserCourses);

	return (
		<div className="flex flex-col px-3 py-2 bg-wood-700 h-full">
			<div className="flex flex-col space-y-3 overflow-y-auto h-full hide-scroll">
				<Link href="/home" className="mb-0">
					<IconJourney className="mx-auto w-7 h-7" />
				</Link>
				<hr className="border-ivory/10 my-3" />
				{courses.data?.map((course) => (
					<Link href={`/course/${course.id}`} className="sweep-up" key={course.id}>
						<div
							className="bg-wood-300 w-0.5 h-8 absolute left-1.5 rounded-full"
							style={{ display: location.startsWith(`/course/${course.id}`) ? "block" : "none" }}
						/>
						<SidebarIcon
							iconStyle={SidebarIconStyle.WOOD}
							style={{
								backgroundColor: course.colour ?? undefined,
							}}
						>
							{course.name[0]}
						</SidebarIcon>
					</Link>
				))}
			</div>
			{/* <div className="mt-auto pt-2">
				<SidebarIcon iconStyle={SidebarIconStyle.WOOD}>
					<IconCog className="w-6 h-6 mx-auto" />
				</SidebarIcon>
			</div> */}
		</div>
	);
};
