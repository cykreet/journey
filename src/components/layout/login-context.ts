import { createContext } from "react";
import type { AuthStatus as AuthStatusPayload } from "../../bindings";

export interface LoginContext {
	showDialog: boolean;
	// auth status here refers to the last authentication attempt status
	// from the current session
	authStatus?: AuthStatusPayload;
	loading?: boolean;
	timeoutId?: number;
	// timerRef: React.MutableRefObject<number | undefined>;
	setShowDialog: (show: boolean) => void;
	setAuthStatus: (status: AuthStatusPayload | undefined) => void;
	setLoading: (loading: boolean) => void;
	setTimeoutId: (id: number | undefined) => void;
}

export const LoginContext = createContext<LoginContext | null>(null);
