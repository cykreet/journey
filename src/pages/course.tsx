import { useRoute } from "wouter";
import { commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import { useCommand } from "../hooks/useCommand";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data, error, loading } = useCommand(commands.getCourse, Number(params?.id));

	if (!match || error) return <div>{error}</div>;
	const pageId = params?.page;
	const currentItem = data?.sections.flatMap((section) => section.items).find((item) => item.id === Number(pageId));
	const sidebarSections = data?.sections?.map((section) => ({
		id: section.section.id,
		name: section.section.name,
		subItems: section.items.map((item) => ({
			name: item.name,
			href: `/course/${params?.id}/${item.id}`,
		})),
	})) as MenuSidebarSection[];

	return (
		<MenuLayout
			header={<span className="font-bold">{data?.course.name}</span>}
			sidebarSections={sidebarSections}
			loading={loading}
		>
			<h2>{currentItem?.name}</h2>
		</MenuLayout>
	);
};
