import type { ShortcutAction } from "../types/input";
import type { ShortcutConfig } from "./input";

export interface AppSettings {
	notesFolder: string;
	locale: string;
	theme: string;
	rememberAppLayout: boolean;
	notesListLayout: "list" | "masonry";
	rememberSettings: boolean;
	searchMode: "notes" | "threads" | "tags";
	searchIsFuzzy: boolean;
	searchSelectedTag: string | null;
	sidebarOpen: boolean;
	controlCenterWidth: number;
	defaultThreadName: string | null;
	useDefaultThreadName: boolean;
	identiconStyle: "dotmatrix" | "round" | "none";
	threadShortcutsMode: "navigation" | "actions";
	dateFormatStyle: "medium" | "narrow";
	textCopyMode: "plain" | "markdown";
	floatingToolbarEnabled: boolean;
	allowManualNoteCreation: boolean;
	shortcuts: Partial<Record<ShortcutAction, ShortcutConfig>>;
	fontFamily: string | null;
	useCustomFont: boolean;
}

export interface ThemeInfo {
	id: string;
	name: string;
}
