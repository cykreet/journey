import { IconChartCircles } from "@tabler/icons-react";

export const Loading = () => {
	return (
		<div className="flex justify-center items-center w-full h-full my-auto">
			<IconChartCircles className="animate-spin text-goo w-8 h-8" />
		</div>
	);
};
