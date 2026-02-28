import { invoke } from "@tauri-apps/api/core";
import { appState, locale, settings, translations } from "$lib";

export const initializeApp = async () => {
	try {
		const initialState: {
			notes_folder: string | null;
			locale: string;
			translations: Record<string, string>;
			today_note_path: string | null;
			today_note_content: string | null;
		} = await invoke("initialize_app");

		if (initialState.notes_folder) {
			translations.set(initialState.translations);
			locale.set(initialState.locale);
			settings.set({
				notes_folder: initialState.notes_folder,
				locale: initialState.locale,
			});
		}

		appState.update((state) => ({
			...state,
			todayNotePath: initialState.today_note_path,
			todayNoteContent: initialState.today_note_content,
		}));
	} catch (error) {
		console.error("Failed to initialize app:", error);
	}
};
