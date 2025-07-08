import { useEffect, useRef, useState } from "react";
import type { Result } from "../bindings";

export interface Command<T> {
	data?: T;
	error?: string;
	loading?: boolean;
}

function useDeepCompareEffect(callback: () => void, dependencies: any[]) {
	const currentDependenciesRef = useRef<any[]>();
	if (!currentDependenciesRef.current || !deepCompare(currentDependenciesRef.current, dependencies)) {
		currentDependenciesRef.current = dependencies;
	}

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(callback, [currentDependenciesRef.current]);
}

function deepCompare(a: any[], b: any[]): boolean {
	return JSON.stringify(a) === JSON.stringify(b);
}

export const useCommand = <T>(
	command: (...args: any) => Promise<Result<T, unknown>>,
	...args: Parameters<typeof command>
): Command<T> => {
	const [commandValue, setCommandValue] = useState<T | undefined>();
	const [error, setError] = useState<string | undefined>();
	const [loading, setLoading] = useState(true);

	useDeepCompareEffect(() => {
		const executeCommand = async () => {
			setLoading(true);
			setError(undefined);

			try {
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
				setLoading(false);
			}
		};

		executeCommand();
	}, [command, ...args]);

	return { data: commandValue, error, loading };
};
