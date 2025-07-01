import { useEffect, useState } from "react";
import type { Result } from "../bindings";

export interface Command<T> {
	data?: T;
	error?: string;
}

export const useCommand = <T>(
	command: (...args: any) => Promise<Result<T, unknown>>,
	...args: Parameters<typeof command>
): Command<T> => {
	const [commandValue, setCommandValue] = useState<T | undefined>();

	useEffect(() => {
		const getUserCourses = async () => {
			const result = await command(...args);
			if (result.status !== "ok") {
				console.log(result.error);
				return { error: result.error };
			}

			setCommandValue(result.data);
		};

		getUserCourses();
	}, [command, args]);

	return { data: commandValue };
};
