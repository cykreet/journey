import { createContext } from "react";

export interface ModuleContext {
	name?: string | null;
	loading: boolean;
	setName: (name?: string) => void;
	setLoading: (loading: boolean) => void;
}

export const ModuleContext = createContext<ModuleContext | null>(null);
