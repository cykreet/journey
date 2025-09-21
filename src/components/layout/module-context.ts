import { createContext } from "react";
import type { SyncError } from "../../bindings";

export interface ModuleContext {
	name?: string | null;
	error?: SyncError | null;
	loading: boolean;
	setName: (name?: string) => void;
	setError: (error?: SyncError) => void;
	setLoading: (loading: boolean) => void;
}

export const ModuleContext = createContext<ModuleContext | null>(null);
