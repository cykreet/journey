import type React from "react";
import { Sidebar } from "./sidebar";

export const GlobalLayout = ({ children }: { children: React.ReactNode }) => {
	return (
		<div className="flex flex-row max-h-svh overflow-hidden w-full">
			<Sidebar />
			<div className="w-full">
				<div>{children}</div>
			</div>
		</div>
	);
};
