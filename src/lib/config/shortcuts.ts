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
	addTag: {
		key: "t",
		ctrl: true,
		description: "Add tag",
	},
	removeTag: {
		key: "t",
		ctrl: true,
		shift: true,
		description: "Remove tag",
	},
	closePopup: {
		key: "Escape",
		description: "Close popup",
	},
};
