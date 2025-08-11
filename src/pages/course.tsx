import { appLocalDataDir, join } from "@tauri-apps/api/path";
import prettyMs from "pretty-ms";
import React, { useEffect, useState } from "react";
import { useRoute } from "wouter";
import { navigate } from "wouter/use-browser-location";
import { type CourseWithSections, commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";
import { useCommand } from "../hooks/useCommand";
import { convertFileSrc } from "@tauri-apps/api/core";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data: courseData, error, loading } = useCommand(commands.getCourse, Number(params?.id));
	const [localAppDataDir, setLocalAppDataDir] = useState<string | null>(null);

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

	useEffect(() => {
		if (params?.page == null && courseData?.sections[0]?.items[0] && courseData?.course.id === Number(params?.id)) {
			const lastViewedModule = localStorage.getItem(`lastViewedModule-${params.id}`);
			if (lastViewedModule == null)
				return navigate(`/course/${params.id}/${courseData?.sections[0]?.items[0]?.id}`, { replace: true });
			return navigate(`/course/${params.id}/${lastViewedModule}`, { replace: true });
		}
	}, [params.page, courseData, params.id]);

	useEffect(() => {
		appLocalDataDir().then((dir) => {
			setLocalAppDataDir(dir);
		});
	}, []);

	return (
		<MenuLayout
			key={params.id}
			header={<span className="font-bold">{courseData?.course.name}</span>}
			sidebarSections={sidebarSections}
			loading={courseData == null && loading}
		>
			{params.page != null && localAppDataDir && (
				<CourseModule
					localAppDataDir={localAppDataDir}
					courseData={courseData as CourseWithSections}
					courseId={Number(params.id)}
					pageId={Number(params.page)}
				/>
			)}
		</MenuLayout>
	);
};

export const CourseModule = ({
	localAppDataDir,
	courseId,
	pageId,
}: { localAppDataDir: string; courseData: CourseWithSections; courseId: number; pageId: number }) => {
	const { data: moduleData, error: contentError, loading } = useCommand(commands.getModuleContent, courseId, pageId);
	const [contentParsed, setContentParsed] = useState<{ id: number; content: string }[] | undefined>(undefined);

	if (contentError) {
		return <div className="text-wood-200">No content available for this module.</div>;
	}

	const [module, moduleContent] = moduleData || [];
	const lastSyncTime = Date.now() - (typeof module?.updatedAt === "number" ? Number(module.updatedAt) * 1000 : 0);
	const prettySyncTime = prettyMs(lastSyncTime, {
		compact: false,
		verbose: true,
		hideSeconds: true,
	});

	useEffect(() => {
		if (moduleData == null || loading) return;
		localStorage.setItem(`lastViewedModule-${courseId}`, pageId.toString());
	}, [moduleData, courseId, pageId, loading]);

	useEffect(() => {
		if (moduleContent == null || localAppDataDir == null) return;
		const parseContent = async () => {
			const parsedContent = await Promise.all(
				moduleContent.map(async (content) => {
					const contentHtml = await replaceSrcAsync(content.content);
					return { id: content.id, content: contentHtml };
				}),
			);
			setContentParsed(parsedContent);
		};

		parseContent();
	}, [moduleContent, localAppDataDir]);

	const replaceSrcAsync = async (html: string) => {
		const regex = /src="([^"]+)"/g;
		const matches = Array.from(html.matchAll(regex));
		let replacedHtml = html;

		for (const match of matches) {
			const srcPath = match[1];
			if (srcPath.startsWith("http://") || srcPath.startsWith("https://") || srcPath.startsWith("data:")) continue;
			const filePath = await join(localAppDataDir, "content_blobs", pageId.toString(), srcPath);
			const localPath = convertFileSrc(filePath);
			replacedHtml = replacedHtml.replace(match[0], `src="${localPath}"`);
		}

		return replacedHtml;
	};

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
					{contentParsed?.map((content) => (
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
