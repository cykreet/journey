import { convertFileSrc } from "@tauri-apps/api/core";
import prettyMs from "pretty-ms";
import { memo, useEffect, useState } from "react";
import { useRoute } from "wouter";
import { navigate } from "wouter/use-browser-location";
import {
	type ContentBlob,
	type CourseWithSections,
	type ModuleContent,
	type SectionModule,
	commands,
} from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";
import { useCommand } from "../hooks/useCommand";
import { SectionModuleType } from "../types";

export const Course = () => {
	const [match, params] = useRoute("/course/:courseId/:moduleId?");
	const { data: courseData, error: _error, loading } = useCommand(commands.getCourse, Number(params?.courseId));

	if (!match || params.courseId == null) navigate("/home", { replace: true });
	if (params?.courseId == null) throw new Error("course id not found in params");

	console.log("module count", courseData?.course.moduleCount);
	let moduleOmitCount = courseData?.course.moduleCount ?? 0;
	const sidebarSections = courseData?.sections?.map((section) => {
		moduleOmitCount -= section.modules.length;
		console.log("subtracting from module omit count", section.modules.length, "now", moduleOmitCount);

		return {
			id: section.section.id,
			name: section.section.name,
			subItems: section.modules.map((courseModule) => ({
				name: courseModule.name,
				href: `/course/${params?.courseId}/${courseModule.id}`,
			})),
		};
	}) as MenuSidebarSection[];

	useEffect(() => {
		if (
			params?.moduleId == null &&
			courseData?.sections[0]?.modules[0] &&
			courseData?.course.id === Number(params?.courseId)
		) {
			const lastViewedModule = localStorage.getItem(`lastViewedModule-${params.courseId}`);
			if (lastViewedModule == null)
				return navigate(`/course/${params.courseId}/${courseData?.sections[0]?.modules[0]?.id}`, { replace: true });
			return navigate(`/course/${params.courseId}/${lastViewedModule}`, { replace: true });
		}
	}, [params.moduleId, courseData, params.courseId]);

	return (
		<MenuLayout
			key={params.courseId}
			header={<span className="font-bold">{courseData?.course.name}</span>}
			sidebarSections={sidebarSections}
			loading={courseData == null && loading}
			sidebarNotice={moduleOmitCount > 0 ? `Omitted ${moduleOmitCount} unsupported modules` : undefined}
		>
			{params.moduleId != null && (
				<CourseModule
					key={params.moduleId}
					courseData={courseData as CourseWithSections}
					courseId={Number(params.courseId)}
					moduleId={Number(params.moduleId)}
				/>
			)}
		</MenuLayout>
	);
};

const CourseModule = ({
	courseId,
	moduleId,
}: { courseData: CourseWithSections; courseId: number; moduleId: number }) => {
	const { data, error, loading } = useCommand(commands.getModuleContent, courseId, moduleId);

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
		localStorage.setItem(`lastViewedModule-${courseId}`, moduleId.toString());
	}, [data, courseId, moduleId, loading]);

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
			{(error && (
				<div className="mb-4">
					<span className="font-bold">Error loading module content:</span> {error}
				</div>
			)) ||
				(data && <ModuleContentBlock courseId={courseId} moduleData={moduleData!} moduleContent={moduleContent} />)}
		</div>
	);
};

const ModuleContentBlock = ({
	courseId,
	moduleData,
	moduleContent,
}: { courseId: number; moduleData: SectionModule; moduleContent?: ModuleContent[] }) => {
	const { data: contentBlobs } = useCommand(commands.getContentBlobs, courseId, moduleData.id);

	if (moduleContent == null || moduleContent.length === 0) {
		// todo: replace with prettier errors
		return <div className="text-wood-100">No content available for this module.</div>;
	}

	if (moduleData.moduleType === SectionModuleType.Resource) {
		return <ResourceContentBlock moduleData={moduleData} contentBlobs={contentBlobs} />;
	}

	if (moduleData.moduleType === SectionModuleType.Page || moduleData.moduleType === SectionModuleType.Book) {
		return <PageContentBlock moduleContent={moduleContent} contentBlobs={contentBlobs} />;
	}
};

const ResourceContentBlock = ({
	contentBlobs,
	moduleData,
}: { contentBlobs?: ContentBlob[]; moduleData: SectionModule }) => {
	const contentBlob = contentBlobs?.[0];
	const localPath = convertFileSrc(contentBlob?.path ?? "");

	if (contentBlob?.mimeType === "application/pdf") {
		return (
			<div>
				<object
					data={localPath}
					type="application/pdf"
					aria-label={moduleData.name}
					className="w-full h-screen object-contain outline-none"
				/>
			</div>
		);
	}
};

const PageContentBlock = memo(
	({ moduleContent, contentBlobs }: { moduleContent: ModuleContent[]; contentBlobs?: ContentBlob[] }) => {
		const [contentBlocks, setContentBlocks] = useState<{ id: number; content: string }[] | undefined>(undefined);

		useEffect(() => {
			// todo: parsed src strings are not used when loading for some reason
			const parseContent = async () => {
				const parsedContent = moduleContent.map((content) => {
					// todo: compile latex expressions using something like katex
					const contentHtml = replaceSrc(content.content);
					return { id: content.id, content: contentHtml };
				});

				setContentBlocks(parsedContent);
			};

			parseContent();

			return () => {
				setContentBlocks(undefined);
			};
		}, [moduleContent]);

		const replaceSrc = (html: string) => {
			const regex = /src="([^"]+)"/g;
			const matches = Array.from(html.matchAll(regex));
			let replacedHtml = html;

			for (const match of matches) {
				const srcPath = match[1];
				if (srcPath.startsWith("http://") || srcPath.startsWith("https://") || srcPath.startsWith("data:")) continue;
				const filePath = contentBlobs?.find((blob) => decodeURI(srcPath).includes(blob.name))?.path;
				if (filePath == null) continue;

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
	},
);
