import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { NoteListResponse } from "$lib/interfaces/notes";

/**
 * Opens a native file dialog to let the user select a directory for their notes.
 */
export const selectFolder = async () => {
	try {
		const selected = await open({
			directory: true,
			multiple: false,
			title: "Select a folder",
		});
		return selected as string | null;
	} catch (error) {
		console.error("Error selecting folder:", error);
		return null;
	}
};

/**
 * Fetches a list of notes available in the currently configured notes folder.
 * If a limit is provided, only the most recent N notes are processed.
 */
export const listNotes = async (
	limit?: number,
): Promise<NoteListResponse | null> => {
	try {
		const response = (await invoke("list_notes", {
			limit,
		})) as NoteListResponse;
		return response;
	} catch (error) {
		console.error("Frontend: Error listing notes:", error);
		return null;
	}
};
