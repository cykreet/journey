-- Create "course_item" table
CREATE TABLE `course_item` (`id` integer NULL PRIMARY KEY AUTOINCREMENT, `course_id` int NOT NULL, `title` varchar NOT NULL, `content_type` int NOT NULL);
-- Create "course_content" table
CREATE TABLE `course_content` (`id` integer NULL PRIMARY KEY AUTOINCREMENT, `course_id` int NOT NULL, `name` varchar NOT NULL);
