import type { NoteContentResponse } from "./notes";
import type { PopupType } from "./ui";

export type AppState = {
	todayNotePath: string | null;
	todayNoteContent: NoteContentResponse | null;
	activePopup: PopupType;
};
