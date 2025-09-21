import { useEffect } from "react";

export function useOnClickOutside(ref: React.RefObject<HTMLElement>, handler: () => void) {
	useEffect(() => {
		const listener = (event: MouseEvent | TouchEvent) => {
			if (!ref.current || ref.current.contains(event.target as Node)) return;
			handler();
		};

		const onKeyPress = (event: KeyboardEvent) => {
			if (event.key !== "Escape") return;
			handler();
		};

		document.addEventListener("mousedown", listener);
		document.addEventListener("touchstart", listener);
		document.addEventListener("keydown", onKeyPress);

		return () => {
			document.removeEventListener("mousedown", listener);
			document.removeEventListener("touchstart", listener);
			document.removeEventListener("keydown", onKeyPress);
		};
	}, [ref, handler]);
}
