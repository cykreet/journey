import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import IconJourney from "~icons/journey/journey";
import { type AuthStatus as AuthStatusPayload, commands } from "../bindings";
import { Button } from "../components/button";
import { Dialog, DialogBodyFailed, DialogBodySuccess } from "../components/dialog";
import { Input } from "../components/input";
import { AuthStatus, SessionStatus } from "../types";
import { getVersion } from "@tauri-apps/api/app";
import { navigate } from "wouter/use-browser-location";
import IconArrowRight from "~icons/tabler/arrow-right";
import { useSessionStatus } from "../hooks/useSessionStatus";

export const Index = () => {
	const [showDialog, setShowDialog] = useState(false);
	const [authStatus, setAuthStatus] = useState<AuthStatusPayload>();
	const [loading, setLoading] = useState(false);
	const [host, setHost] = useState("");
	const [version, setVersion] = useState("");
	const sessionStatus = useSessionStatus();

	const timerRef = useRef<number>();
	const authStateRef = useRef<AuthStatusPayload>();

	useEffect(() => {
		if (sessionStatus === SessionStatus.Valid) return navigate("/home");
	}, [sessionStatus]);

	useEffect(() => {
		getVersion().then(setVersion);
		authStateRef.current = authStatus;
	}, [authStatus]);

	useEffect(() => {
		const unlistenPromise = listen<AuthStatusPayload>("moodle_auth", (event) => {
			setLoading(false);
			if (event.payload !== AuthStatus.Aborted) setShowDialog(true);
			setAuthStatus(event.payload);
			setAuthTimeout(event.payload);
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearTimeout(timerRef.current);
		};
	}, []);

	const openLoginWindow = async () => {
		if (!host[0] || loading) return;
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
				// if (state === AuthStatus.Aborted) return;
				if (state === AuthStatus.Success) return navigate("/home");
				setAuthStatus(undefined);
				setShowDialog(false);
			},
			3000,
			payload,
		);
	};

	return (
		<React.Fragment>
			{showDialog && (
				<Dialog open={showDialog}>
					{authStatus === AuthStatus.Success && <DialogBodySuccess message="Successfully authenticated." />}
					{authStatus === AuthStatus.Failed && <DialogBodyFailed message="Failed to authenticate." />}
				</Dialog>
			)}
			<div className="flex flex-col justify-center items-center h-screen space-y-4 bg-wood-700">
				<div className="flex flex-row mx-auto max-w-1/2 container space-x-10 items-center">
					<IconJourney className="w-34 h-34" />
					<div className="flex flex-col space-y-2">
						<div className="flex flex-row space-x-2">
							<h1>Journey</h1>
							<div className="text-sm border border-ivory/10 rounded-md p-1 text-wood-100">v{version}</div>
						</div>
						<span>
							Get started by authenticating with your Moodle instance.
							<a
								className="text-sm ml-0.5 text-wood-100 align-top"
								title="Learn more"
								href="https://github.com/cykreet/journey"
								target="_blank"
								rel="noreferrer"
							>
								?
							</a>
						</span>
					</div>
				</div>
				<div className="flex flex-col w-full max-w-1/3">
					<span className="text-sm text-wood-100">Enter the host of your Moodle instance here.</span>
					<div className="flex flex-row space-x-2 w-full items-center">
						<Input
							className="w-full"
							type="url"
							disabled={loading}
							onChange={(value) => {
								if (!value[0]) return;

								try {
									const parsed = new URL(value);
									setHost(parsed.protocol + parsed.host);
								} catch {
									setHost("");
								}
							}}
							onEnter={openLoginWindow}
							placeholder="https://moodle.example.com"
						/>
						<Button onClick={openLoginWindow} loading={loading} disabled={!host[0]}>
							<IconArrowRight className="w-6 h-6" />
						</Button>
					</div>
				</div>
			</div>
		</React.Fragment>
	);
};
