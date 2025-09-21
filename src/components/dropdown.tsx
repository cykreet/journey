import { createContext, useContext, useEffect, useRef, useState } from "react";
import { useOnClickOutside } from "../hooks/click-outside";
import { Button, type ButtonProps, ButtonStyle } from "./button";

const DropdownContext = createContext<{ open: boolean; setOpen: (open: boolean) => void } | null>(null);

function useDropdownContext() {
	const context = useContext(DropdownContext);
	if (!context) throw new Error("Dropdown components must be used within a Dropdown");
	return context;
}

export interface DropdownItemProps extends ButtonProps {}

function DropdownDivider() {
	return <div className="border-t border-border" />;
}

function DropdownItem({ children, onClick, ...rest }: DropdownItemProps) {
	const { setOpen } = useDropdownContext();

	const handleClick = () => {
		onClick?.();
		setOpen(false);
	};

	return (
		<Button className="text-sm w-full text-left" onClick={handleClick} buttonStyle={ButtonStyle.GHOST} {...rest}>
			{children}
		</Button>
	);
}

function DropdownItemContainer({ children }: { children: React.ReactNode }) {
	return <div className="w-full text-ellipsis overflow-hidden text-sm p-2">{children}</div>;
}

function DropdownTrigger({ children }: { children: React.ReactNode }) {
	const { open, setOpen } = useDropdownContext();

	return (
		// biome-ignore lint/a11y/useKeyWithClickEvents: <explanation>
		<div className="cursor-pointer" onClick={() => setOpen(!open)}>
			{children}
		</div>
	);
}

function DropdownMenu({ children }: { children: React.ReactNode }) {
	const { open } = useDropdownContext();
	const dropdownRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (open == false) return;
		const dropdown = dropdownRef.current;
		const dropdownRight = dropdown?.getBoundingClientRect().right ?? 0;
		const windowLimit = window.innerWidth - 20;
		if (dropdown && dropdownRight > windowLimit) {
			dropdown.style.left = `${window.innerWidth - dropdownRight - 20}px`;
		}
	}, [open]);

	if (open == false) return null;

	return (
		<div
			ref={dropdownRef}
			className="absolute rounded-md flex flex-col space-y-1 w-40 bg-steel-700 border border-border"
		>
			{children}
		</div>
	);
}

function Dropdown({ children }: { children?: React.ReactNode }) {
	const [open, setOpen] = useState(false);
	const containerRef = useRef<HTMLDivElement>(null);

	useOnClickOutside(containerRef, () => setOpen(false));

	return (
		<DropdownContext.Provider value={{ open, setOpen }}>
			<div className="relative">
				<div ref={containerRef} className="relative">
					{children}
				</div>
			</div>
		</DropdownContext.Provider>
	);
}

export default Object.assign(Dropdown, {
	Item: DropdownItem,
	ItemContainer: DropdownItemContainer,
	Trigger: DropdownTrigger,
	Menu: DropdownMenu,
	Divider: DropdownDivider,
});
