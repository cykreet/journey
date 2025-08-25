import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useContext, useEffect, useState } from "react";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import { useRoute } from "wouter";
import { navigate } from "wouter/use-browser-location";
import IconJourney from "~icons/journey/journey?color=red";
import {
	type ContentBlob,
	type CourseWithSections,
	type ModuleContent,
	type SectionModule,
	commands,
} from "../bindings";
import { MenuLayout } from "../components/layout/menu/menu-layout";
import type { MenuSidebarSection } from "../components/layout/menu/menu-sidebar";
import { ModuleContext } from "../components/layout/module-context";
import { useCommand } from "../hooks/useCommand";
import { SectionModuleType } from "../types";

const SRC_REGEX = /src="([^"]+)"/g;
const ANCHOR_REGEX = /<a[^>](.[^>]+)>/g;

pdfjs.GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/build/pdf.worker.min.mjs", import.meta.url).toString();
const pdfOptions = {
	wasmUrl: "/wasm/",
	cMapUrl: "/cmaps/",
	standardFontDataUrl: "/standard_fonts/",
};

export const Course = () => {
	const [match, params] = useRoute("/course/:courseId/:moduleId?");
	const { data: courseData, error: _error, loading } = useCommand(commands.getCourse, Number(params?.courseId));

	if (!match || params.courseId == null) navigate("/home", { replace: true });
	if (params?.courseId == null) throw new Error("course id not found in params");

	let moduleOmitCount = courseData?.course.moduleCount ?? 0;
	let selectedModuleName: string | undefined;
	const sidebarSections = courseData?.sections?.map((section) => {
		moduleOmitCount -= section.modules.length;

		return {
			id: section.section.id,
			name: section.section.name,
			subItems: section.modules.map((courseModule) => {
				if (courseModule.id.toString() === params?.moduleId) selectedModuleName = courseModule.name;

				return {
					name: courseModule.name,
					href: `/course/${params?.courseId}/${courseModule.id}`,
				};
			}),
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
					selectedModuleName={selectedModuleName}
					courseData={courseData as CourseWithSections}
					courseId={Number(params.courseId)}
					moduleId={Number(params.moduleId)}
				/>
			)}
		</MenuLayout>
	);
};

const CourseModule = ({
	selectedModuleName,
	courseId,
	moduleId,
}: { selectedModuleName?: string; courseData: CourseWithSections; courseId: number; moduleId: number }) => {
	const { data, error, loading } = useCommand(commands.getModuleContent, courseId, moduleId);
	const { data: contentBlobs, loading: blobsLoading } = useCommand(commands.getContentBlobs, courseId, moduleId);
	const moduleContext = useContext(ModuleContext);

	const [moduleData, moduleContent] = data || [];

	useEffect(() => {
		if (data == null || loading) return;
		localStorage.setItem(`lastViewedModule-${courseId}`, moduleId.toString());
	}, [data, courseId, moduleId, loading]);

	useEffect(() => {
		if (moduleContext == null) return;
		moduleContext.setName(selectedModuleName);
		moduleContext.setLoading((loading || blobsLoading) ?? true);
		moduleContext.setError(error);

		return () => {
			moduleContext.setName(undefined);
			moduleContext.setLoading(false);
			moduleContext.setError(undefined);
		};
	}, [moduleContext, selectedModuleName, loading, blobsLoading, error]);

	if ((moduleData == null || moduleContent == null) && loading == false) return;
	if (moduleData == null || moduleContent == null) return;

	if (moduleData.moduleType === SectionModuleType.Resource) {
		return <ResourceContentBlock moduleData={moduleData} contentBlobs={contentBlobs} />;
	}

	if (moduleData.moduleType === SectionModuleType.Page || moduleData.moduleType === SectionModuleType.Book) {
		return <PageContentBlock moduleContent={moduleContent} contentBlobs={contentBlobs} />;
	}
};

const ResourceContentBlock = ({
	contentBlobs,
	// moduleData,
}: { contentBlobs?: ContentBlob[]; moduleData: SectionModule }) => {
	// todo: move these to a separate component
	const [pageCount, setPageCount] = useState<number | undefined>(undefined);

	const contentBlob = contentBlobs?.[0];
	const localPath = convertFileSrc(contentBlob?.path ?? "");

	if (contentBlob?.mimeType === "application/pdf") {
		return (
			<Document
				file={localPath}
				options={pdfOptions}
				className="items-center w-full h-full justify-center flex flex-col space-y-4"
				loading={<IconJourney className="w-14 h-14 mt-10 text-wood-300" />}
				onLoadSuccess={({ numPages }) => setPageCount(numPages)}
				externalLinkRel="noreferrer noopener"
				externalLinkTarget="_blank"
			>
				{[...Array(pageCount).keys()].map((index) => (
					<Page renderAnnotationLayer renderTextLayer key={index + 1} pageNumber={index + 1} width={800} />
				))}
			</Document>
		);
	}
};

const PageContentBlock = ({
	moduleContent,
	contentBlobs,
}: { moduleContent: ModuleContent[]; contentBlobs?: ContentBlob[] }) => {
	const [contentBlocks, setContentBlocks] = useState<{ id: number; content: string }[] | undefined>(undefined);

	useEffect(() => {
		const parseContent = async () => {
			const parsedContent = moduleContent.map((content) => {
				// todo: compile latex expressions using something like katex
				let contentHtml = replaceSrc(contentBlobs ?? [], content.content);
				contentHtml = fixAnchors(contentHtml);
				return { id: content.id, content: contentHtml };
			});

			setContentBlocks(parsedContent);
		};

		parseContent();

		return () => {
			setContentBlocks(undefined);
		};
	}, [contentBlobs, moduleContent]);

	const replaceSrc = useCallback((blobs: ContentBlob[], html: string) => {
		const matches = Array.from(html.matchAll(SRC_REGEX));
		let replacedHtml = html;

		for (const match of matches) {
			const srcPath = match[1];
			if (srcPath.startsWith("http://") || srcPath.startsWith("https://") || srcPath.startsWith("data:")) continue;
			const filePath = blobs.find((blob) => decodeURI(srcPath).includes(blob.name))?.path;
			if (filePath == null) continue;

			const localPath = convertFileSrc(filePath);
			replacedHtml = replacedHtml.replace(match[0], `src="${localPath}"`);
		}

		return replacedHtml;
	}, []);

	const fixAnchors = useCallback((html: string) => {
		const matches = Array.from(html.matchAll(ANCHOR_REGEX));
		let replacedHtml = html;

		for (const match of matches) {
			if (match[1].includes("target=")) continue;
			replacedHtml = replacedHtml.replace(match[1], `${match[1]} target="_blank" rel="noreferrer"`);
		}

		return replacedHtml;
	}, []);

	return (
		<div className="mt-10 mb-20 xs:max-w-[48rem] max-w-[40rem] mx-auto flex flex-col space-y-4" id="module-content">
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
