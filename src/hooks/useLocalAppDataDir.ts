import { appLocalDataDir } from "@tauri-apps/api/path";
import { useState, useEffect } from "react";

export function useLocalAppDataDir() {
	const [localAppDataDir, setLocalAppDataDir] = useState<string>("");

	useEffect(() => {
		appLocalDataDir().then((dir) => {
			if (dir == null) throw new Error("Failed to get local app data directory");
			setLocalAppDataDir(dir);
		});
	}, []);

	return localAppDataDir;
}
