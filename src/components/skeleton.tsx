import clsx from "clsx";
import type { CSSProperties } from "react";

export function Skeleton({ className, style }: { className?: string; style?: CSSProperties }) {
	const classNames = clsx("h-6 animate-pulse bg-ivory/8 rounded", className);
	return <div className={classNames} style={style} />;
}
