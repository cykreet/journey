import clsx from "clsx";

export function Card({ children, className }: { title?: string; className?: string; children: React.ReactNode }) {
	const classes = clsx(
		"bg-steel-700 border-b-6 border border-ivory/10 border-b-ivory/10 rounded-lg px-5 py-4",
		className,
	);

	return <div className={classes}>{children}</div>;
}
