// most of these are derived from types in bindings.ts, but enums are more convenient
export enum AuthStatus {
	Failed = "Failed",
	Success = "Success",
	Aborted = "Aborted",
	Pending = "Pending",
}

export enum SectionModuleType {
	Page = "page",
	Book = "book",
	Forum = "forum",
	Resource = "resource",
	Unknown = "Unknown",
}
