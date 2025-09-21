import { useContext, useEffect } from "react";
import { events, commands } from "../bindings";
import type { AuthStatus as AuthStatusPayload } from "../bindings";
import { LoginContext } from "../components/layout/login-context";
import { AuthStatus } from "../types";
import { useLocation } from "wouter";

export function useLoginWindow() {
	const loginContext = useContext(LoginContext);
	const [location, navigate] = useLocation();

	useEffect(() => {
		if (loginContext == null) throw new Error("LoginContext has not been set");
		const unlistenPromise = events.moodleAuthEvent.listen((event) => {
			loginContext.setLoading(false);
			if (event.payload !== AuthStatus.Aborted) loginContext.setShowDialog(true);
			loginContext.setAuthStatus(event.payload);
			console.log("setting timeout", event.payload);
			setAuthTimeout(event.payload);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearTimeout(loginContext.timeoutId);
		};
	}, [loginContext]);

	const openLoginWindow = async (host: string) => {
		if (!host[0] || loginContext?.loading) return;
		loginContext?.setLoading(true);
		const loginResult = await commands.openLoginWindow(host);

		if (loginResult.status === "error") {
			loginContext?.setLoading(false);
			loginContext?.setShowDialog(true);
			loginContext?.setAuthStatus(AuthStatus.Failed);
			setAuthTimeout(AuthStatus.Failed);
		}
	};

	const setAuthTimeout = (payload: AuthStatusPayload) => {
		if (loginContext?.timeoutId) clearTimeout(loginContext.timeoutId);
		if (loginContext == null) return;
		loginContext.setTimeoutId(
			setTimeout(
				(state: AuthStatusPayload) => {
					// if (state === AuthStatus.Aborted) return;
					// loginContext?.setAuthStatus(undefined);
					loginContext?.setShowDialog(false);
					if (state === AuthStatus.Success && location === "/") navigate("/home");
					else if (state === AuthStatus.Success) navigate(location);
				},
				3000,
				payload,
			),
		);
	};

	return { openLoginWindow, loading: loginContext?.loading };
}
