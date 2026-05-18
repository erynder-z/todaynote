import type { PopupType } from "../types/ui";
import type { LocaleInfo } from "./locale";
import type { NoteContentResponse, ThreadAggregationResult } from "./notes";
import type { ThemeInfo } from "./settings";

export interface SessionState {
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	aggregatedThread: ThreadAggregationResult | null;
	activePopup: PopupType;
	isMac: boolean;
	sidebarOpen: boolean;
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
	availableLocales: LocaleInfo[];
	availableThemes: ThemeInfo[];
	translations: Record<string, string>;
	themeColors: Record<string, string>;
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	isMac: boolean;
	controlCenterWidth: number;
}
