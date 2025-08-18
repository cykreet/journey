import { type LinkProps, Link as WouterLink } from "wouter";
import { openUrl } from "@tauri-apps/plugin-opener";

export const Link = (props: LinkProps) => {
	if (props.href?.startsWith("http")) {
		return (
			<WouterLink
				href={props.href}
				rel="noreferrer"
				target="_blank"
				onClick={(event) => {
					event.preventDefault();
					openUrl(props.href);
				}}
			/>
		);
	}

	return <WouterLink {...props} />;
};
