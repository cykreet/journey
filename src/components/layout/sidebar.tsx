import { Link, useLocation } from "wouter";
import IconJourney from "~icons/journey/journey";
import { commands } from "../../bindings";
import { useCommand } from "../../hooks/command";
import { SidebarIcon } from "./sidebar-icon";

export function Sidebar() {
	const [location] = useLocation();
	const courses = useCommand(commands.getUserCourses);

	return (
		<div className="flex flex-col py-2 bg-steel-700 h-full min-w-14">
			<div className="flex flex-col space-y-3 overflow-y-auto h-full hide-scroll w-full">
				<Link href="/home" className="mb-0">
					<IconJourney className="mx-auto w-7 h-7" />
				</Link>
				<hr className="border-border w-8 mx-auto my-3" />
				{courses.data?.map((course) => (
					<Link href={`/course/${course.id}`} className="sweep-up inline-flex relative w-full" key={course.id}>
						<div
							className="w-0.5 h-8 rounded-full absolute"
							style={{
								display: location.startsWith(`/course/${course.id}`) ? "block" : "none",
								backgroundColor: course.colour ?? undefined,
							}}
						/>
						<SidebarIcon
							title={course.name}
							className="mx-auto"
							style={{
								backgroundColor: course.colour ?? undefined,
								color: course.colour != null ? "var(--color-steel)" : undefined,
							}}
						>
							{course.name[0]}
						</SidebarIcon>
					</Link>
				))}
			</div>
		</div>
	);
}
