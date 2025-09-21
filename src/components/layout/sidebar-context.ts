import { createContext } from "react";

export interface SidebarContext {
	collapsed: boolean;
	setCollapsed: (collapsed: boolean) => void;
}

export const SidebarContext = createContext<SidebarContext | null>(null);
