export type ShortcutCallback = (
	e: KeyboardEvent,
) => boolean | void | Promise<void>;

export type ShortcutAction =
	| "toggleSearch"
	| "toggleFindInNote"
	| "toggleNoteBrowser"
	| "toggleSettings"
	| "toggleStatistics"
	| "toggleSidebar"
	| "toggleNoteBrowserLayout"
	| "manageTags"
	| "closePopup"
	| "focusLastLine"
	| "jumpByNumber"
	| "toggleFuzzy"
	| "toggleSearchMode"
	| "navigateYesterday"
	| "navigateLastAvailable"
	| "navigateToday"
	| "toggleThreadOptionsMode"
	| "threadOptionRemove"
	| "threadOptionLinked"
	| "toggleBold"
	| "toggleItalic"
	| "toggleStrikethrough"
	| "toggleCode"
	| "toggleBlockquote"
	| "toggleLink"
	| "copySelection";
