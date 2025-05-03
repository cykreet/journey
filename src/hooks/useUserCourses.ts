import { useEffect, useState } from "react";
import { commands, type Course } from "../bindings";

export const useUserCourses = (): Course[] | undefined => {
	const [coursesValue, setCoursesValue] = useState<Course[] | undefined>();

	useEffect(() => {
		const getUserCourses = async () => {
			const courses = await commands.getUserCourses();
			if (courses.status !== "ok") return null;
			setCoursesValue(courses.data);
		};

		getUserCourses();
	}, []);

	return coursesValue;
};
