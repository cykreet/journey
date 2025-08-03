import { useRoute } from "wouter";
import { commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import { useCommand } from "../hooks/useCommand";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data, error, loading } = useCommand(commands.getCourse, Number(params?.id));
	const {
		data: moduleData,
		error: contentError,
		loading: contentLoading,
	} = useCommand(commands.getModuleContent, Number(params?.id), Number(params?.page));

	if (!match || error) return <div>{error}</div>;
	// const pageId = params?.page;
	// const currentItem = data?.sections.flatMap((section) => section.items).find((item) => item.id === Number(pageId));
	const [module, moduleContent] = moduleData || [];
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
			<div className="flex flex-row items-center text-wood-100 justify-between mb-4 bg-wood-700 rounded border border-ivory/10 p-2">
				<span className="font-bold">{module?.name}</span>
			</div>
			{moduleContent?.map((content) => (
				<div key={content.id} className="h-full w-full">
					{/* biome-ignore lint/security/noDangerouslySetInnerHtml: <explanation> */}
					<div style={{ height: "100%" }} dangerouslySetInnerHTML={{ __html: content.content }} />
				</div>
			))}
		</MenuLayout>
	);
};
