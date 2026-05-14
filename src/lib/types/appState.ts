import type { LocaleInfo } from "./locale";
import type { NoteContentResponse, ThreadAggregationResult } from "./notes";
import type { ThemeInfo } from "./settings";
import type { PopupType } from "./ui";

export type SessionState = {
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	aggregatedThread: ThreadAggregationResult | null;
	activePopup: PopupType;
	isMac: boolean;
	sidebarOpen: boolean;
};

export type AppPayload = {
	notesFolder: string | null;
	locale: string;
	theme: string;
	rememberWindowSize: boolean;
	notesListLayout: "list" | "masonry";
	availableLocales: LocaleInfo[];
	availableThemes: ThemeInfo[];
	translations: Record<string, string>;
	themeColors: Record<string, string>;
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	isMac: boolean;
};
