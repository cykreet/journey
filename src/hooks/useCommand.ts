import { useEffect, useState } from "react";
import type { Result } from "../bindings";

export interface Command<T> {
	data?: T;
	error?: string;
	loading?: boolean;
}

export const useCommand = <T>(
	command: (...args: any) => Promise<Result<T, unknown>>,
	...args: Parameters<typeof command>
): Command<T> => {
	const [commandValue, setCommandValue] = useState<T | undefined>();
	const [error, setError] = useState<string | undefined>();
	const [loading, setLoading] = useState(true);

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		let cancelled = false;
		const executeCommand = async () => {
			setLoading(true);
			setError(undefined);

			try {
				if (cancelled) return;
				const result = await command(...args);
				if (result.status !== "ok") {
					setError(result.error as string);
					setCommandValue(undefined);
				} else {
					setCommandValue(result.data);
					setError(undefined);
				}
			} catch (err) {
				setError(String(err));
				setCommandValue(undefined);
			} finally {
				if (!cancelled) setLoading(false);
			}
		};

		executeCommand();
		return () => {
			cancelled = true;
		};
	}, [command, ...args]);

	return { data: commandValue, error, loading };
};
