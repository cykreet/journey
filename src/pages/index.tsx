import { useState } from "react";
import IconJourney from "~icons/journey/journey";
import IconArrowRight from "~icons/tabler/arrow-right";
import { Button } from "../components/button";
import { Input } from "../components/input";
import { Link } from "../components/link";
import { useLoginWindow } from "../hooks/login-window";
import { useVersion } from "../hooks/version";

export function Index() {
	const { openLoginWindow, loading } = useLoginWindow();
	const [host, setHost] = useState("");
	const version = useVersion();

	return (
		<div className="flex flex-col justify-center items-center w-full space-y-6">
			<div className="flex flex-row space-x-10 items-center justify-center">
				<IconJourney className="w-30 h-30 text-accent" />
				<div className="flex flex-col space-y-2 w-min">
					<div className="flex flex-row space-x-2 w-fit">
						<h1>Journey</h1>
						<div className="text-sm border border-ivory/10 rounded-md p-1 text-steel-100">v{version}</div>
					</div>
					<span className="w-60">
						Get started by authenticating with your Moodle instance.
						<Link
							className="text-sm ml-0.5 text-steel-100 align-top"
							title="Learn more"
							href="https://github.com/cykreet/journey"
						>
							?
						</Link>
					</span>
				</div>
			</div>
			<div className="flex flex-col w-full max-w-1/4">
				<span className="text-sm text-steel-100">Enter the host of your Moodle instance here.</span>
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
						onEnter={() => openLoginWindow(host)}
						placeholder="https://moodle.example.com"
					/>
					<Button onClick={() => openLoginWindow(host)} loading={loading} disabled={!host[0]} className="px-4">
						<IconArrowRight className="w-6 h-6" />
					</Button>
				</div>
			</div>
		</div>
	);
}
