import type { ShortcutAction } from "../types/input";
import type { PopupType } from "../types/ui";
import type { ShortcutConfig } from "./input";
import type { LocaleInfo } from "./locale";
import type {
	NoteContentResponse,
	NoteThread,
	ThreadAggregationResult,
} from "./notes";
import type { ThemeInfo } from "./settings";

export interface SessionState {
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	aggregatedThread: ThreadAggregationResult | null;
	activePopup: PopupType;
	isMac: boolean;
	sidebarOpen: boolean;
	selectedThreadForOptions: NoteThread | null;
	threadShortcutsMode: "navigation" | "actions";
}

export interface AppPayload {
	notesFolder: string | null;
	locale: string;
	theme: string;
	rememberAppLayout: boolean;
	notesListLayout: "list" | "masonry";
	rememberSettings: boolean;
	searchMode: "notes" | "threads" | "tags";
	searchIsFuzzy: boolean;
	searchSelectedTag: string | null;
	sidebarOpen: boolean;
	defaultThreadName: string | null;
	identiconStyle: "dotmatrix" | "round" | "none";
	threadShortcutsMode: "navigation" | "actions";
	availableLocales: LocaleInfo[];
	availableThemes: ThemeInfo[];
	translations: Record<string, string>;
	themeColors: Record<string, string>;
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	isMac: boolean;
	controlCenterWidth: number;
	shortcuts: Partial<Record<ShortcutAction, ShortcutConfig>>;
}
