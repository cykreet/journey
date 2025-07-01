import { commands } from "../bindings";
import { useCommand } from "../hooks/useCommand";

export const Index = () => {
	const userSession = useCommand(commands.getUserSession);

	return (
		<div>
			<span>session: {userSession.data ?? "none"}</span>
		</div>
	);
};
