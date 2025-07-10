-- Create "course" table
CREATE TABLE `course` (
  `id` integer NULL,
  `name` varchar NOT NULL,
  `colour` varchar NULL,
  `icon` varchar NULL,
  PRIMARY KEY (`id`)
);
-- Create "course_section" table
CREATE TABLE `course_section` (
  `id` integer NULL,
  `course_id` int NOT NULL,
  `name` varchar NOT NULL,
  `updated_at` int NULL DEFAULT (strftime('%s', 'now')),
  `items` text NULL,
  PRIMARY KEY (`id`)
);
