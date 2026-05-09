export type PopupType =
	| "folderSelector"
	| "noteBrowser"
	| "search"
	| "tagManager"
	| "shortcuts"
	| null;

export type ShortcutHint = {
	label: string;
	key?: string;
};

export type ToastType = "info" | "success" | "warning" | "error";

export type Toast = {
	id: string;
	message: string;
	type: ToastType;
	duration?: number;
};
