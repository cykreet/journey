import React from "react";
import { Loading } from "./loading";
import IconCircleCheckFilled from "~icons/tabler/circle-check-filled";
import IconCircleXFilled from "~icons/tabler/circle-x-filled";
import { Card } from "./card";

export const Dialog = ({
	open,
	loading,
	children,
	title,
}: { open?: boolean; title?: string; loading?: boolean; children: React.ReactNode }) => {
	if (open === false) return null;

	return (
		<div className="w-full h-full flex justify-center items-center inset-0 fixed bg-wood-700/50">
			<Card className="sweep-up min-h-20 space-y-2 flex flex-col w-md">
				{(loading && <Loading decorated iconClassName="w-8 h-8" />) || (
					<React.Fragment>
						{title && <span className="font-bold">{title}</span>}
						{children}
					</React.Fragment>
				)}
			</Card>
		</div>
	);
};

export const DialogBodyFailed = ({ message }: { message: string }) => {
	return (
		<div className="flex flex-col space-y-2 justify-center items-center my-auto">
			<IconCircleXFilled className="mx-auto text-wood-300 w-8 h-8" />
			<span>{message}</span>
		</div>
	);
};

export const DialogBodySuccess = ({ message }: { message: string }) => {
	return (
		<div className="flex flex-col space-y-2 justify-center items-center my-auto">
			<IconCircleCheckFilled className="mx-auto text-goo w-8 h-8" />
			<span>{message}</span>
		</div>
	);
};
