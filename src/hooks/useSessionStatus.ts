import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { SessionStatus } from "../types";
import { debug } from "@tauri-apps/plugin-log";

export function useSessionStatus() {
	const [sessionStatus, setSessionStatus] = useState<SessionStatus>(SessionStatus.Unknown);

	useEffect(() => {
		const unlistenPromise = listen<SessionStatus>("session_change", (event) => {
			debug(`Session status changed: ${event.payload}`);
			setSessionStatus(event.payload);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, []);

	return sessionStatus;
}
