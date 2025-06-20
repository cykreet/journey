import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import { type AuthStatus, commands } from "../../bindings";
import { Button, ButtonStyle } from "../button";
import { Dialog, DialogBodyFailed, DialogBodySuccess } from "../dialog";
import { Input } from "../input";
import { Sidebar } from "./sidebar";

export const GlobalLayout = ({ children }: { children: React.ReactNode }) => {
	const [showDialog, setShowDialog] = useState(false);
	const [authState, setAuthState] = useState<AuthStatus>();
	const [loading, setLoading] = useState(false);
	const [host, setHost] = useState("");
	const timerRef = useRef<number>();
	const authStateRef = useRef<AuthStatus>();

	useEffect(() => {
		const unlistenPromise = listen<AuthStatus>("login_closed", (event) => {
			setLoading(false);
			setAuthState(event.payload);
			timerRef.current = setTimeout(
				(state: AuthStatus) => {
					if (state === "Aborted") return;
					setAuthState(undefined);
					setShowDialog(false);
				},
				3000,
				event.payload,
			);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearTimeout(timerRef.current);
		};
	}, []);

	useEffect(() => {
		authStateRef.current = authState;
	}, [authState]);

	const openLoginWindow = async () => {
		setLoading(true);
		commands.openLoginWindow(host);
	};

	return (
		<React.Fragment>
			<Dialog open={showDialog} loading={loading} title={authState == null ? "Moodle Host" : undefined}>
				{(authState === "Failed" && <DialogBodyFailed message="Failed to authenticate, please try again." />) ||
					(authState === "Success" && <DialogBodySuccess message="Successfully logged in." />) || (
						<React.Fragment>
							Enter the host URL of your Moodle instance here.
							<Input className="mt-4" type="url" onChange={setHost} placeholder="https://moodle.example.com" />
							<div className="flex flex-row space-x-4 ml-auto mt-4">
								<Button onClick={openLoginWindow}>Confirm</Button>
								<Button buttonStyle={ButtonStyle.IVORY} onClick={() => setShowDialog(false)}>
									Cancel
								</Button>
							</div>
						</React.Fragment>
					)}
			</Dialog>
			<div className="flex flex-row max-h-svh overflow-hidden w-full">
				<Sidebar onUserClick={() => setShowDialog(true)} />
				<div className="w-full">{children}</div>
			</div>
		</React.Fragment>
	);
};
