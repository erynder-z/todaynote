import type { ShortcutAction, ShortcutConfig } from "$lib/types/input";

export const defaultShortcuts: Record<ShortcutAction, ShortcutConfig> = {
	toggleSearch: {
		key: "k",
		primary: true,
		description: "Toggle search",
	},
	toggleNotesList: {
		key: "l",
		primary: true,
		description: "Toggle notes list",
	},
	toggleSettings: {
		key: ",",
		primary: true,
		description: "Toggle settings",
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
		key: "i",
		primary: true,
		description: "Focus last line",
	},
	jumpByNumber: {
		key: "1,2,3,4,5,6,7,8,9,a,b,c,d,e,f,g,h,i,j,k",
		primary: true,
		secondary: true,
		description: "Jump to section",
	},
	toggleFuzzy: {
		key: "f",
		primary: true,
		secondary: true,
		description: "Toggle fuzzy search",
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
		"KeyA",
		"KeyB",
		"KeyC",
		"KeyD",
		"KeyE",
		"KeyF",
		"KeyG",
		"KeyH",
		"KeyI",
		"KeyJ",
		"KeyK",
	],
	labels: "123456789ABCDEFGHIJK".split(""),
};
