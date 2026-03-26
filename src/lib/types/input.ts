export type ShortcutCallback = (e: KeyboardEvent) => void | Promise<void>;

export type ShortcutAction =
	| "toggleSearch"
	| "toggleNotesList"
	| "toggleSettings"
	| "manageTags"
	| "closePopup";

export type ShortcutConfig = {
	key: string;
	primary?: boolean;
	secondary?: boolean;
	description?: string;
};

export type ShortcutRegistration = ShortcutConfig & {
	callback: ShortcutCallback;
};
