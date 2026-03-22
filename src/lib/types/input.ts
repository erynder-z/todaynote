export type ShortcutCallback = (e: KeyboardEvent) => void | Promise<void>;

export type ShortcutAction =
	| "toggleSearch"
	| "toggleNotesList"
	| "toggleSettings"
	| "addTag"
	| "removeTag"
	| "closePopup";

export type ShortcutConfig = {
	key: string;
	ctrl?: boolean;
	shift?: boolean;
	alt?: boolean;
	meta?: boolean;
	description?: string;
};

export type ShortcutRegistration = ShortcutConfig & {
	callback: ShortcutCallback;
};
