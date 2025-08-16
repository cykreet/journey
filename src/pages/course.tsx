import { join } from "@tauri-apps/api/path";
import prettyMs from "pretty-ms";
import { useEffect, useState } from "react";
import { useRoute } from "wouter";
import { navigate } from "wouter/use-browser-location";
import { type CourseWithSections, type ModuleContent, type SectionModule, commands } from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";
import { useCommand } from "../hooks/useCommand";
import { convertFileSrc } from "@tauri-apps/api/core";
import { SectionModuleType } from "../types";
import { useLocalAppDataDir } from "../hooks/useLocalAppDataDir";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const { data: courseData, error: _error, loading } = useCommand(commands.getCourse, Number(params?.id));

	if (!match || params.id == null) navigate("/home", { replace: true });
	if (params?.id == null) throw new Error("course id not found in params");

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

	return (
		<MenuLayout
			key={params.id}
			header={<span className="font-bold">{courseData?.course.name}</span>}
			sidebarSections={sidebarSections}
			loading={courseData == null && loading}
		>
			{params.page != null && (
				<CourseModule
					courseData={courseData as CourseWithSections}
					courseId={Number(params.id)}
					pageId={Number(params.page)}
				/>
			)}
		</MenuLayout>
	);
};

const CourseModule = ({ courseId, pageId }: { courseData: CourseWithSections; courseId: number; pageId: number }) => {
	const { data, error, loading } = useCommand(commands.getModuleContent, courseId, pageId);

	const statusColour = loading ? "bg-wood-100" : error ? "bg-rose-500" : "bg-goo";
	const [moduleData, moduleContent] = data || [];
	const lastSyncTime =
		Date.now() - (typeof moduleData?.updatedAt === "number" ? Number(moduleData.updatedAt) * 1000 : 0);
	const prettySyncTime = prettyMs(lastSyncTime, {
		compact: false,
		verbose: true,
		hideSeconds: true,
	});

	useEffect(() => {
		if (data == null || loading) return;
		localStorage.setItem(`lastViewedModule-${courseId}`, pageId.toString());
	}, [data, courseId, pageId, loading]);

	return (
		<div className="mb-4 w-full">
			<div className="flex flex-col text-wood-100 mb-10 bg-wood-700 rounded border border-ivory/10 p-2">
				<span className="font-bold">{moduleData?.name}</span>
				<div className="flex flex-row space-x-2 items-center">
					<div className={`${statusColour} rounded-full w-1.5 h-1.5 ${loading ? "animate-pulse" : ""}`} />
					<span className="text-sm text-wood-200">
						{loading || error
							? "..."
							: `Last synced ${lastSyncTime > 60 * 1000 ? `${prettySyncTime} ago` : "just now"}`}
					</span>
				</div>
			</div>
			{((data == null || error) && (
				<div className="mb-4">
					<span className="font-bold">Error loading module content:</span> {error}
				</div>
			)) || <ModuleContentBlock pageId={pageId} moduleData={moduleData!} moduleContent={moduleContent} />}
		</div>
	);
};

const ModuleContentBlock = ({
	pageId,
	moduleData,
	moduleContent,
}: { pageId: number; moduleData: SectionModule; moduleContent?: ModuleContent[] }) => {
	if (moduleContent == null || moduleContent.length === 0) {
		// todo: replace with prettier errors
		return <div className="text-wood-100">No content available for this module.</div>;
	}

	if (moduleData.moduleType === SectionModuleType.Resource) {
		return <ResourceContentBlock pageId={pageId} moduleData={moduleData} moduleContent={moduleContent} />;
	}

	if (moduleData.moduleType === SectionModuleType.Page || moduleData.moduleType === SectionModuleType.Book) {
		return <PageContentBlock pageId={pageId} moduleContent={moduleContent} />;
	}

	return <> </>;
};

const ResourceContentBlock = ({
	pageId,
	moduleData,
	moduleContent,
}: { pageId: number; moduleData: SectionModule; moduleContent: ModuleContent[] }) => {
	const localAppDataDir = useLocalAppDataDir();
	const [filePath, setFilePath] = useState<string | null>(null);
	const resourceBlock = moduleContent[0];
	const extension = resourceBlock.content.substring(resourceBlock.content.lastIndexOf(".") + 1);

	useEffect(() => {
		// resource content should just be set to a file path
		join(localAppDataDir, "content_blobs", pageId.toString(), resourceBlock.content).then((path) => {
			const localPath = convertFileSrc(path);
			setFilePath(localPath);
		});

		return () => {
			setFilePath(null);
		};
	}, [pageId, resourceBlock, localAppDataDir]);

	if (extension === "pdf") {
		return (
			<div>
				<object
					data={filePath}
					type="application/pdf"
					aria-label={moduleData.name}
					className="w-full h-screen object-contain outline-none"
				/>
			</div>
		);
	}
};

const PageContentBlock = ({ moduleContent, pageId }: { moduleContent: ModuleContent[]; pageId: number }) => {
	const localAppDataDir = useLocalAppDataDir();
	const [contentBlocks, setContentBlocks] = useState<{ id: number; content: string }[] | undefined>(undefined);

	useEffect(() => {
		const parseContent = async () => {
			const parsedContent = await Promise.all(
				moduleContent.map(async (content) => {
					// todo: compile latex expressions using something like katex
					const contentHtml = await replaceSrcAsync(content.content);
					return { id: content.id, content: contentHtml };
				}),
			);

			setContentBlocks(parsedContent);
		};

		parseContent();

		return () => {
			setContentBlocks(undefined);
		};
	}, [moduleContent]);

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
		<div className="w-full h-full" id="module-content">
			{contentBlocks?.map((block) => (
				<div
					className="w-full h-full"
					key={block.id}
					/* biome-ignore lint/security/noDangerouslySetInnerHtml: <explanation> */
					dangerouslySetInnerHTML={{ __html: block.content }}
				/>
			))}
		</div>
	);
};
