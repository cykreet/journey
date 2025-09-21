import { getVersion } from "@tauri-apps/api/app";
import { useState, useEffect } from "react";

export function useVersion() {
	const [version, setVersion] = useState("");

	useEffect(() => {
		getVersion().then(setVersion);
	}, []);

	return version;
}
