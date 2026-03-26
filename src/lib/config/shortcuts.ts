import type { ShortcutAction, ShortcutConfig } from "$lib/types/input";

export const defaultShortcuts: Record<ShortcutAction, ShortcutConfig> = {
	toggleSearch: {
		key: "k",
		ctrl: true,
		description: "Toggle search",
	},
	toggleNotesList: {
		key: "l",
		ctrl: true,
		description: "Toggle notes list",
	},
	toggleSettings: {
		key: ",",
		ctrl: true,
		description: "Toggle settings",
	},
	manageTags: {
		key: "t",
		ctrl: true,
		description: "Manage tags",
	},
	closePopup: {
		key: "Escape",
		description: "Close popup",
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
