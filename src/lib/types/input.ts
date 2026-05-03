export type ShortcutCallback = (
	e: KeyboardEvent,
) => boolean | void | Promise<void>;

export type ShortcutAction =
	| "toggleSearch"
	| "toggleNotesList"
	| "toggleSettings"
	| "manageTags"
	| "closePopup"
	| "focusLastLine"
	| "jumpByNumber"
	| "toggleFuzzy";

export type ShortcutConfig = {
	key: string;
	primary?: boolean;
	secondary?: boolean;
	shift?: boolean;
	description?: string;
};

export type ShortcutRegistration = ShortcutConfig & {
	callback: ShortcutCallback;
};
