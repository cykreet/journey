import { useRoute } from "wouter";
import { CourseLayout } from "../components/layout/course/course-layout";
import { useUserCourses } from "../hooks/useUserCourses";
import type { CourseSidebarItem } from "../components/layout/course/course-sidebar";

export const Course = () => {
	const [match, params] = useRoute("/course/:id/:page?");
	const courses = useUserCourses();
	const courseId = params?.id;
	const pageId = params?.page;

	const course = courses?.find((course) => course.id === Number(courseId));
	if (!match || !course) return <div>course not found</div>;

	const coursePages: CourseSidebarItem[] = [
		{
			id: 1,
			name: "Introduction",
		},
	];

	return (
		<CourseLayout course={course} sidebarItems={coursePages}>
			<h2>Introduction</h2>
			<span>
				course id: {courseId}, page name: {pageId}
			</span>
		</CourseLayout>
	);
};
