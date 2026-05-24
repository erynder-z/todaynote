import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import { sessionState } from "../stores/sessionState.svelte";
import { toast } from "../stores/toast.svelte";
import { t } from "./i18n";
import { readNoteContent } from "./notes";

/**
 * Navigates to a note with the specified date offset.
 *
 * Positive offset goes forward in time, negative offset goes backward.
 * - offset = 0: today
 * - offset = -1: yesterday
 * - offset = -7: one week ago
 * - offset = 7: one week from now
 */
export const navigateToOffset = async (offset: number) => {
	const path = await getNotePathByOffset(offset);
	if (!path) {
		toast.error(get(t)("notes.error.not_found"));
		return null;
	}

	const content = await readNoteContent(path);
	if (content) {
		sessionState.todayNotePath = path;
		sessionState.todayNoteContent = content;
		return content;
	}

	toast.error(get(t)("notes.error.not_found"));
	return null;
};

/**
 * Navigates to the most recent note that is not today's note.
 */
export const navigateToLastAvailable = async () => {
	const path = (await invoke("get_last_available_note_path")) as string | null;
	if (!path) {
		toast.error(get(t)("notes.error.not_found"));
		return null;
	}

	const content = await readNoteContent(path);
	if (content) {
		sessionState.todayNotePath = path;
		sessionState.todayNoteContent = content;
		return content;
	}

	toast.error(get(t)("notes.error.not_found"));
	return null;
};

/**
 * Retrieves the absolute path to a note file offset from today.
 *
 * Positive offset goes forward in time, negative offset goes backward.
 * - offset = 0: today
 * - offset = -1: yesterday
 * - offset = -7: one week ago
 */
export const getNotePathByOffset = async (offset: number) => {
	try {
		const path = (await invoke("get_note_path_by_offset", {
			offset,
		})) as string;
		return path;
	} catch (error) {
		console.error(`Error getting note path for offset ${offset}:`, error);
		return null;
	}
};
