import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useRef, useState } from "react";
import { navigate } from "wouter/use-browser-location";
import IconJourney from "~icons/journey/journey";
import IconArrowRight from "~icons/tabler/arrow-right";
import { commands, events, type AuthStatus as AuthStatusPayload } from "../bindings";
import { Button } from "../components/button";
import { Dialog, DialogBodyFailed, DialogBodySuccess } from "../components/dialog";
import { Input } from "../components/input";
import { Link } from "../components/link";
import { AuthStatus } from "../types";

export function Index() {
	const [showDialog, setShowDialog] = useState(false);
	const [authStatus, setAuthStatus] = useState<AuthStatusPayload>();
	const [loading, setLoading] = useState(false);
	const [host, setHost] = useState("");
	const [version, setVersion] = useState("");

	const timerRef = useRef<number>();
	const authStateRef = useRef<AuthStatusPayload>();

	useEffect(() => {
		getVersion().then(setVersion);
		authStateRef.current = authStatus;
	}, [authStatus]);

	useEffect(() => {
		const unlistenPromise = events.moodleAuthEvent.listen((event) => {
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
		<>
			{showDialog && (
				<Dialog open={showDialog}>
					{authStatus === AuthStatus.Success && <DialogBodySuccess message="Successfully authenticated." />}
					{authStatus === AuthStatus.Failed && <DialogBodyFailed message="Failed to authenticate." />}
				</Dialog>
			)}
			<div className="flex flex-col justify-center items-center w-full space-y-6">
				<div className="flex flex-row space-x-10 items-center justify-center">
					<IconJourney className="w-30 h-30 text-goo" />
					<div className="flex flex-col space-y-2 w-min">
						<div className="flex flex-row space-x-2 w-fit">
							<h1>Journey</h1>
							<div className="text-sm border border-ivory/10 rounded-md p-1 text-wood-100">v{version}</div>
						</div>
						<span className="w-60">
							Get started by authenticating with your Moodle instance.
							<Link
								className="text-sm ml-0.5 text-wood-100 align-top"
								title="Learn more"
								href="https://github.com/cykreet/journey"
							>
								?
							</Link>
						</span>
					</div>
				</div>
				<div className="flex flex-col w-full max-w-1/4">
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
									setHost(`${parsed.protocol}//${parsed.host}`);
								} catch {
									setHost("");
								}
							}}
							onEnter={openLoginWindow}
							placeholder="https://moodle.example.com"
						/>
						<Button onClick={openLoginWindow} loading={loading} disabled={!host[0]} className="px-4">
							<IconArrowRight className="w-6 h-6" />
						</Button>
					</div>
				</div>
			</div>
		</>
	);
}
