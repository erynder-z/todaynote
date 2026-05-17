import type { ShortcutConfig } from "$lib/interfaces/input";
import type { ShortcutAction } from "$lib/types/input";

export const defaultShortcuts: Record<ShortcutAction, ShortcutConfig> = {
	toggleSearch: {
		key: "k",
		primary: true,
		description: "Toggle search",
	},
	toggleNoteBrowser: {
		key: "l",
		primary: true,
		description: "Toggle note browser",
	},
	toggleSettings: {
		key: ",",
		primary: true,
		description: "Toggle settings",
	},
	toggleNoteBrowserLayout: {
		key: "l",
		primary: true,
		secondary: true,
		description: "Toggle note browser layout",
	},
	manageTags: {
		key: "t",
		primary: true,
		description: "Manage tags",
	},
	closePopup: {
		key: "Escape",
		description: "Close popup",
	},
	focusLastLine: {
		key: "0",
		primary: true,
		secondary: true,
		description: "Focus last line",
	},
	jumpByNumber: {
		key: "1,2,3,4,5,6,7,8,9,b,c,d,g,h,i,j,k,n,p,r",
		primary: true,
		secondary: true,
		description: "Jump to thread",
	},
	toggleFuzzy: {
		key: "f",
		primary: true,
		secondary: true,
		description: "Toggle fuzzy search",
	},
	toggleSearchMode: {
		key: "m",
		primary: true,
		secondary: true,
		description: "Toggle search mode",
	},
	navigateYesterday: {
		key: "e",
		primary: true,
		secondary: true,
		description: "Go to yesterday's note",
	},
	navigateLastAvailable: {
		key: "a",
		primary: true,
		secondary: true,
		description: "Go to last available note",
	},
	navigateToday: {
		key: "o",
		primary: true,
		secondary: true,
		description: "Go to today's note",
	},
};

export const tagSuggestionShortcuts = {
	codes: [
		"Digit1",
		"Digit2",
		"Digit3",
		"Digit4",
		"Digit5",
		"Digit6",
		"Digit7",
		"Digit8",
		"Digit9",
		"KeyB",
		"KeyC",
		"KeyD",
		"KeyG",
		"KeyH",
		"KeyI",
		"KeyJ",
		"KeyK",
		"KeyN",
		"KeyP",
		"KeyR",
	],
	labels: "123456789BCDGHIKNPR".split(""),
};
