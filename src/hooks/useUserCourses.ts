import { useEffect, useState } from "react";
import type { Result } from "../bindings";

export const useCommand = <T>(command: () => Promise<Result<T, unknown>>): T | undefined => {
	const [commandValue, setCommandValue] = useState<T | undefined>();

	useEffect(() => {
		const getUserCourses = async () => {
			const result = await command();
			// todo: handle error somehow
			if (result.status !== "ok") {
				console.log(result.error);
				return null;
			}

			console.log(result.data);
			setCommandValue(result.data);
		};

		getUserCourses();
	}, [command]);

	return commandValue;
};
