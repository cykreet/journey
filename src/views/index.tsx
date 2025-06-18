import { commands } from "../bindings";
import { useCommand } from "../hooks/useUserCourses";

export const Index = () => {
	const userSession = useCommand(commands.getUserSession);

	return (
		<div className="flex container mx-auto flex-col mt-10">
			<h1>Index</h1>
			<span>session: {userSession ?? "none"}</span>
		</div>
	);
};
