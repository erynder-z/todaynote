export type PopupType =
	| "folderSelector"
	| "notesList"
	| "search"
	| "tagManager"
	| "shortcuts"
	| null;

export type ShortcutHint = {
	label: string;
	key?: string;
};
