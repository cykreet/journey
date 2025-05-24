import React from "react";
import { Loading } from "./loading";
import { IconCircleCheckFilled, IconCircleXFilled } from "@tabler/icons-react";

export const Dialog = ({
	open,
	loading,
	children,
	title,
}: { open: boolean; title?: string; loading?: boolean; children: React.ReactNode }) => {
	if (!open) return null;

	return (
		<div className="w-full h-full flex justify-center items-center absolute bg-wood-700/50">
			<div className="bg-wood-700 border-b-6 border border-ivory/10 border-b-ivory/10 min-h-20 rounded-lg px-5 py-4 space-y-2 flex flex-col w-md">
				{(loading && <Loading />) || (
					<React.Fragment>
						{title && <span className="font-bold">{title}</span>}
						{children}
					</React.Fragment>
				)}
			</div>
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
