import clsx from "clsx";
import IconLoader from "~icons/tabler/loader-2";

export function Loading({ iconClassName, decorated }: { iconClassName?: string; decorated?: boolean }) {
	const classes = clsx("animate-spin", decorated && "text-accent", iconClassName);

	return (
		<div className="flex justify-center items-center">
			<IconLoader className={classes} />
		</div>
	);
}
