import { createContext } from "react";

export interface ModuleContext {
	name?: string | null;
	loading: boolean;
	error?: string | null;
	setName: (name?: string) => void;
	setLoading: (loading: boolean) => void;
	setError: (error?: string) => void;
}

export const ModuleContext = createContext<ModuleContext | null>(null);
