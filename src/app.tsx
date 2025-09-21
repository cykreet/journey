import "./main.css";
import { useState } from "react";
import { Route, Switch } from "wouter";
import type { AuthStatus as AuthStatusPayload } from "./bindings";
import { GlobalLayout } from "./components/layout/global-layout";
import { LoginContext } from "./components/layout/login-context";
import { WindowControls } from "./components/layout/window-controls";
import { Index } from "./pages";
import { Announcements } from "./pages/announcements";
import { Course } from "./pages/course";
import { Home } from "./pages/home";
import { Dialog, DialogBodySuccess, DialogBodyFailed } from "./components/dialog";
import { AuthStatus } from "./types";

export function App() {
	const [showAuthDialog, setShowAuthDialog] = useState(false);
	const [authLoading, setAuthLoading] = useState(false);
	const [authStatus, setAuthStatus] = useState<AuthStatusPayload>();
	const [timeoutId, setTimeoutId] = useState<number>();
	// const timerRef = useRef<number>();

	return (
		<LoginContext.Provider
			value={{
				showDialog: showAuthDialog,
				setShowDialog: setShowAuthDialog,
				timeoutId: timeoutId,
				authStatus,
				setAuthStatus,
				setTimeoutId: setTimeoutId,
				loading: authLoading,
				setLoading: setAuthLoading,
			}}
		>
			{showAuthDialog && (
				<Dialog onClickOutside={() => setShowAuthDialog(false)}>
					{authStatus === AuthStatus.Success && <DialogBodySuccess message="Successfully authenticated." />}
					{authStatus === AuthStatus.Failed && <DialogBodyFailed message="Failed to authenticate." />}
				</Dialog>
			)}
			<WindowControls>
				<Switch>
					<Route path="/">
						<Index />
					</Route>
					<GlobalLayout>
						<Route path="/home">
							<Home />
						</Route>
						<Route path="/announcements">
							<Announcements />
						</Route>
						<Route path="/course/:id?/:page?">
							<Course />
						</Route>
					</GlobalLayout>
					<Route>
						<p>404 not found</p>
					</Route>
				</Switch>
			</WindowControls>
		</LoginContext.Provider>
	);
}

export default App;
