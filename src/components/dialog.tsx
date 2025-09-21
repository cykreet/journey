import { useRef } from "react";
import IconCircleCheckFilled from "~icons/tabler/circle-check-filled";
import IconCircleXFilled from "~icons/tabler/circle-x-filled";
import { useOnClickOutside } from "../hooks/click-outside";
import { Card } from "./card";
import { Loading } from "./loading";

export function Dialog({
	loading,
	children,
	title,
	onClickOutside,
}: { title?: string; loading?: boolean; onClickOutside?: () => void; children: React.ReactNode }) {
	const dialogRef = useRef<HTMLDivElement>(null);

	useOnClickOutside(dialogRef, () => {
		console.log("clicked outside dialog");
		if (onClickOutside) onClickOutside();
	});

	return (
		<div className="w-full h-full flex justify-center items-center z-10 inset-0 fixed bg-steel-700/50">
			<div ref={dialogRef}>
				<Card className="sweep-up min-h-20 space-y-2 flex flex-col w-md">
					{(loading && <Loading decorated iconClassName="w-8 h-8" />) || (
						<>
							{title && <span className="font-bold">{title}</span>}
							{children}
						</>
					)}
				</Card>
			</div>
		</div>
	);
}

export const DialogBodyFailed = ({ message }: { message: string }) => {
	return (
		<div className="flex flex-col space-y-2 justify-center items-center my-auto">
			<IconCircleXFilled className="mx-auto text-steel-300 w-8 h-8" />
			<span>{message}</span>
		</div>
	);
};

export const DialogBodySuccess = ({ message }: { message: string }) => {
	return (
		<div className="flex flex-col space-y-2 justify-center items-center my-auto">
			<IconCircleCheckFilled className="mx-auto text-accent w-8 h-8" />
			<span>{message}</span>
		</div>
	);
};
