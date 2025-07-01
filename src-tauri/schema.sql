CREATE TABLE `course` (
	`id` INTEGER PRIMARY KEY,
	`name` VARCHAR(100) NOT NULL,
	`colour` VARCHAR(10),
	`icon` VARCHAR(20)
);

CREATE TABLE `course_item` (
	`id` INTEGER PRIMARY KEY AUTOINCREMENT,
	`parent_id` INT(20),
	`course_id` INT(20) NOT NULL,
	`title` VARCHAR(100) NOT NULL,
	`content_type` INT(1) NOT NULL,
	`updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	`content` TEXT
);