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
	| "toggleSearchMode"
	| "navigateYesterday"
	| "navigateLastAvailable"
	| "navigateToday";
