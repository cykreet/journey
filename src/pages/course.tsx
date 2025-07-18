import { useRoute } from "wouter";
import { commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import { useCommand } from "../hooks/useCommand";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data: courses, error } = useCommand(commands.getCourse, Number(params?.id));
	// const { data: sections, loading } = useCommand(commands.getCourseSections, Number(params?.id));

	if (!match || error) return <div>{error}</div>;
	const pageId = params?.page;
	// const currentItem = sections
	// 	?.find((section) => section.items.find((item) => item.id === Number(pageId)))
	// 	?.items.find((item) => item.id === Number(pageId));

	// const sidebarSections = sections?.map((section) => ({
	// 	id: section.id,
	// 	name: section.name,
	// 	subItems: section.items.map((item) => ({
	// 		name: item.name,
	// 		href: `/course/${params?.id}/${item.id}`,
	// 	})),
	// })) as MenuSidebarSection[];

	return (
		<MenuLayout
			header={<span className="font-bold">{courses?.name}</span>}
			// sidebarSections={sidebarSections}
			// loading={loading}
		>
			{/* <h2>{currentItem?.name}</h2> */}
		</MenuLayout>
	);
};
