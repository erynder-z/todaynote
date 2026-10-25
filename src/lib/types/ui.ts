export type PopupType =
	| "settings"
	| "noteBrowser"
	| "noteMetadata"
	| "pinnedThreads"
	| "search"
	| "tagManager"
	| "shortcuts"
	| "threadAggregation"
	| "threadView"
	| "about"
	| "statistics"
	| "threadOptions"
	| "addImage"
	| null;

export type ToastType = "info" | "success" | "warning" | "error";

export type ModalSize = "minimal" | "sm" | "md" | "lg" | "xl" | "full";

export type PurgeReport = {
	removed: number;
	freedBytes: number;
};
