import { useEffect, useState } from "react";
import type { Result } from "../bindings";

export interface Command<T> {
	data?: T;
	error?: string;
	loading?: boolean;
}

export function useCommand<T>(
	command: (...args: any) => Promise<Result<T, unknown>>,
	...args: Parameters<typeof command>
): Command<T> {
	const [commandValue, setCommandValue] = useState<T | undefined>();
	const [error, setError] = useState<string | undefined>();
	const [loading, setLoading] = useState(true);

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		const cacheKey = `command-${command.name}-${JSON.stringify(args)}`;
		const getCachedData = () => {
			try {
				const cachedData = localStorage.getItem(cacheKey);
				if (cachedData == null) return undefined;
				return JSON.parse(cachedData) as T;
			} catch {
				return undefined;
			}
		};

		let cancelled = false;
		const executeCommand = async () => {
			setLoading(true);
			setError(undefined);

			try {
				const result = await command(...args);
				if (cancelled) return;
				if (result.status === "ok") {
					setCommandValue(result.data);
					localStorage.setItem(cacheKey, JSON.stringify(result.data));
					setError(undefined);
				} else {
					setError(result.error as string);
					setCommandValue(undefined);
				}
			} catch (err) {
				setError(String(err));
				setCommandValue(undefined);
			} finally {
				if (!cancelled) setLoading(false);
			}
		};

		setCommandValue(getCachedData());
		executeCommand();
		return () => {
			cancelled = true;
		};
	}, [command, ...args]);

	return { data: commandValue, error, loading };
}
