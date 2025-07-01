import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import IconUserFilled from "~icons/tabler/user-filled";
import { type AuthStatus as AuthStatusPayload, commands } from "../../bindings";
import { AuthStatus } from "../../types";
import { Button, ButtonStyle } from "../button";
import { Dialog, DialogBodyFailed, DialogBodySuccess } from "../dialog";
import { Input } from "../input";
import { SidebarIcon, SidebarIconStyle } from "../layout/sidebar-icon";

export const User = () => {
	const [showDialog, setShowDialog] = useState(false);
	const [authStatus, setAuthStatus] = useState<AuthStatusPayload>();
	const [loading, setLoading] = useState(false);
	const [host, setHost] = useState("");
	const timerRef = useRef<number>();
	const authStateRef = useRef<AuthStatusPayload>();

	useEffect(() => {
		const unlistenPromise = listen<AuthStatusPayload>("login_closed", (event) => {
			setLoading(false);
			setAuthStatus(event.payload);
			setAuthTimeout(event.payload);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearTimeout(timerRef.current);
		};
	}, []);

	useEffect(() => {
		authStateRef.current = authStatus;
	}, [authStatus]);

	const openLoginWindow = async () => {
		setLoading(true);
		const loginResult = await commands.openLoginWindow(host);

		if (loginResult.status === "error") {
			setLoading(false);
			setShowDialog(true);
			setAuthStatus(AuthStatus.Failed);
			setAuthTimeout(AuthStatus.Failed);
		}
	};

	const setAuthTimeout = (payload: AuthStatusPayload) => {
		if (timerRef.current) clearTimeout(timerRef.current);
		timerRef.current = setTimeout(
			(state: AuthStatusPayload) => {
				if (state === AuthStatus.Aborted) return;
				setAuthStatus(undefined);
				setShowDialog(false);
			},
			3000,
			payload,
		);
	};

	return (
		<React.Fragment>
			<Dialog open={showDialog} loading={loading} title={authStatus == null ? "Moodle Login" : undefined}>
				{(authStatus === AuthStatus.Failed && (
					<DialogBodyFailed message="Failed to authenticate, please try again." />
				)) ||
					(authStatus === AuthStatus.Success && <DialogBodySuccess message="Successfully authenticated." />) || (
						<React.Fragment>
							Enter the domain of your Moodle instance here.
							<Input
								className="mt-4"
								type="url"
								onChange={(value) => {
									if (!value[0]) return;
									const parsed = new URL(value);
									setHost(parsed.protocol + parsed.host);
								}}
								onEnter={openLoginWindow}
								placeholder="https://moodle.example.com"
							/>
							<div className="flex flex-row space-x-4 ml-auto mt-4">
								<Button onClick={openLoginWindow}>Confirm</Button>
								<Button buttonStyle={ButtonStyle.IVORY} onClick={() => setShowDialog(false)}>
									Cancel
								</Button>
							</div>
						</React.Fragment>
					)}
			</Dialog>
			<SidebarIcon iconStyle={SidebarIconStyle.GOO} onClick={() => setShowDialog(true)}>
				<IconUserFilled className="mx-auto" />
			</SidebarIcon>
		</React.Fragment>
	);
};
