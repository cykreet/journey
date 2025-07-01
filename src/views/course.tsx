import { useRoute } from "wouter";
import { commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import { useCommand } from "../hooks/useCommand";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data, error } = useCommand(commands.getUserCourse, Number(params?.id));

	if (!match || error) return <div>{error}</div>;
	const pageId = params?.page;

	return (
		<MenuLayout header={<span className="font-bold">{data?.name}</span>}>
			<h2>Introduction</h2>
			<span>
				course id: {params.id}, page name: {pageId}
			</span>
		</MenuLayout>
	);
};
