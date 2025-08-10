import prettyMs from "pretty-ms";
import React, { useEffect } from "react";
import { useRoute } from "wouter";
import { navigate } from "wouter/use-browser-location";
import { type CourseWithSections, commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";
import { useCommand } from "../hooks/useCommand";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data: courseData, error, loading } = useCommand(commands.getCourse, Number(params?.id));

	if (!match || error) return <div>{error}</div>;

	const sidebarSections = courseData?.sections?.map((section) => {
		return {
			id: section.section.id,
			name: section.section.name,
			subItems: section.items.map((item) => ({
				name: item.name,
				href: `/course/${params?.id}/${item.id}`,
			})),
		};
	}) as MenuSidebarSection[];

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		if (loading === false && params?.page == null) {
			const lastViewedModule = localStorage.getItem(`lastViewedModule-${params.id}`);
			if (lastViewedModule == null)
				return navigate(`/course/${params.id}/${courseData?.sections[0]?.items[0]?.id}`, { replace: true });
			return navigate(`/course/${params.id}/${lastViewedModule}`, { replace: true });
		}
	}, [params.id, params.page, loading]);

	return (
		<MenuLayout
			header={<span className="font-bold">{courseData?.course.name}</span>}
			sidebarSections={sidebarSections}
			loading={courseData == null && loading}
		>
			{(courseData || loading === false) && (
				<CourseModule
					courseData={courseData as CourseWithSections}
					courseId={Number(params.id)}
					pageId={Number(params.page)}
				/>
			)}
		</MenuLayout>
	);
};

export const CourseModule = ({
	courseId,
	pageId,
}: { courseData: CourseWithSections; courseId: number; pageId: number }) => {
	const { data: moduleData, error: contentError, loading } = useCommand(commands.getModuleContent, courseId, pageId);

	const [module, moduleContent] = moduleData || [];
	const lastSyncTime = Date.now() - (typeof module?.updatedAt === "number" ? Number(module.updatedAt) * 1000 : 0);
	const prettySyncTime = prettyMs(lastSyncTime, {
		compact: false,
		verbose: true,
		hideSeconds: true,
	});

	useEffect(() => {
		if (module == null || moduleContent == null) return;
		localStorage.setItem(`lastViewedModule-${courseId}`, module.id.toString());
	}, [module, moduleContent, courseId]);

	return (
		<React.Fragment>
			{(contentError && (
				<div className="mb-4">
					<span className="font-bold">Error loading module content:</span> {contentError}
				</div>
			)) || (
				<React.Fragment>
					<div className="flex flex-col text-wood-100 mb-4 bg-wood-700 rounded border border-ivory/10 p-2">
						<span className="font-bold">{module?.name}</span>
						<div className="flex flex-row space-x-2 items-center">
							<div className={`bg-goo rounded-full w-1.5 h-1.5 ${loading ? "animate-pulse" : ""}`} />
							<span className="text-sm text-wood-200">
								{loading ? "..." : `Last synced ${lastSyncTime > 60 * 1000 ? `${prettySyncTime} ago` : "just now"}`}
							</span>
						</div>
					</div>
					{moduleContent?.map((content) => (
						<div key={content.id} className="h-full w-full">
							<div
								style={{ height: "100%" }}
								id="module-content"
								/* biome-ignore lint/security/noDangerouslySetInnerHtml: <explanation> */
								dangerouslySetInnerHTML={{ __html: content.content }}
							/>
						</div>
					))}
				</React.Fragment>
			)}
		</React.Fragment>
	);
};
