import { commands } from "../bindings";
import { useCommand } from "../hooks/useCommand";

export const Home = () => {
	const userSession = useCommand(commands.getUserSession);

	return (
		<div className="flex flex-col">
			<span>session: {userSession.data ?? "none"}</span>
		</div>
	);
};
