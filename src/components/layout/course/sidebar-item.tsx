import type { ReactNode } from "react";

export const SidebarItem = ({ children }: { children: ReactNode }) => {
	return <div className="w-full hover:bg-ivory/10 p-1.5 select-none rounded-md cursor-pointer">{children}</div>;
};
