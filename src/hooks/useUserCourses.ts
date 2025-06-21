import { useEffect, useState } from "react";
import type { Result } from "../bindings";

export interface Command<T> {
	data?: T;
	error?: string;
}

export const useCommand = <T>(command: () => Promise<Result<T, unknown>>): Command<T> => {
	const [commandValue, setCommandValue] = useState<T | undefined>();

	useEffect(() => {
		const getUserCourses = async () => {
			const result = await command();
			if (result.status !== "ok") {
				console.log(result.error);
				return { error: result.error };
			}

			setCommandValue(result.data);
		};

		getUserCourses();
	}, [command]);

	return { data: commandValue };
};
