CREATE TABLE `course` (
	`id` INTEGER PRIMARY KEY,
	`name` VARCHAR(100) NOT NULL,
	`colour` VARCHAR(10),
	`icon` VARCHAR(20)
);

CREATE TABLE `course_section` (
	`id` INTEGER PRIMARY KEY,
	`course_id` INT(20) NOT NULL,
	`name` VARCHAR(100) NOT NULL,
	`updated_at` INT(4) DEFAULT (strftime('%s', 'now')),
	`items` TEXT
)