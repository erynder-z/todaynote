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
	notes_folder: string | null;
	locale: string;
	theme: string;
	remember_window_size: boolean;
	available_locales: LocaleInfo[];
	available_themes: ThemeInfo[];
	translations: Record<string, string>;
	theme_colors: Record<string, string>;
	today_note_path: string | null;
	today_note_content: NoteContentResponse | null;
};
