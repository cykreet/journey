import { useState, useEffect } from "react";
import { commands } from "../bindings";

export function useUser() {
	const [userName, setUserName] = useState<string | null>();
	const [host, setHost] = useState<string | null>();

	useEffect(() => {
		async function fetchUserData() {
			const fetchedUser = await commands.getUserName();
			const fetchedHost = await commands.getHost();
			if (fetchedUser.status !== "error") setUserName(fetchedUser.data);
			if (fetchedHost.status !== "error") setHost(fetchedHost.data);
		}

		fetchUserData();
	}, []);

	return { userName, host };
}
