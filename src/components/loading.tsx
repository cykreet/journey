import clsx from "clsx";
import IconChartCircles from "~icons/tabler/chart-circles";

export const Loading = ({ iconClassName, decorated }: { iconClassName?: string; decorated?: boolean }) => {
	const classes = clsx("animate-spin", decorated && "text-goo", iconClassName);

	return (
		<div className="flex justify-center items-center w-full h-full my-auto">
			<IconChartCircles className={classes} />
		</div>
	);
};
