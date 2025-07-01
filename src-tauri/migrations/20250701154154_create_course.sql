-- Create "course" table
CREATE TABLE `course` (
  `id` integer NULL PRIMARY KEY AUTOINCREMENT,
  `name` varchar NOT NULL,
  `colour` varchar NULL,
  `icon` varchar NULL
);
-- Create "course_item" table
CREATE TABLE `course_item` (
  `id` integer NULL PRIMARY KEY AUTOINCREMENT,
  `parent_id` int NULL,
  `course_id` int NOT NULL,
  `title` varchar NOT NULL,
  `content_type` int NOT NULL,
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `content` text NULL
);
