export type ShortcutCallback = (
	e: KeyboardEvent,
) => boolean | void | Promise<void>;

export type ShortcutAction =
	| "toggleSearch"
	| "toggleNoteBrowser"
	| "toggleSettings"
	| "toggleNoteBrowserLayout"
	| "manageTags"
	| "closePopup"
	| "focusLastLine"
	| "jumpByNumber"
	| "toggleFuzzy"
	| "navigateYesterday"
	| "navigateLastAvailable"
	| "navigateToday";

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
