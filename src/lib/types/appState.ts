import type { LocaleInfo } from "./locale";
import type { NoteContentResponse } from "./notes";
import type { ThemeInfo } from "./settings";
import type { PopupType } from "./ui";

export type SessionState = {
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	activePopup: PopupType;
};

export type AppPayload = {
	notesFolder: string | null;
	locale: string;
	theme: string;
	rememberWindowSize: boolean;
	availableLocales: LocaleInfo[];
	availableThemes: ThemeInfo[];
	translations: Record<string, string>;
	themeColors: Record<string, string>;
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
};
