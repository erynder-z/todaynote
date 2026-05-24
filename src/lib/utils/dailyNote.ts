import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import type { NoteContentResponse } from "$lib/interfaces/notes";
import { sessionState } from "../stores/sessionState.svelte";
import { toast } from "../stores/toast.svelte";
import { t } from "./i18n";

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
	try {
		const content = (await invoke("read_note_by_offset", {
			offset,
		})) as NoteContentResponse;

		sessionState.todayNotePath = content.path;
		sessionState.todayNoteContent = content;
		return content;
	} catch (error) {
		toast.error(get(t)("notes.error.not_found"));
		console.error(error);
		return null;
	}
};

/**
 * Navigates to the most recent note that is not today's note.
 */
export const navigateToLastAvailable = async () => {
	try {
		const content = (await invoke(
			"read_last_available_note",
		)) as NoteContentResponse | null;

		if (!content) {
			toast.error(get(t)("notes.error.not_found"));
			return null;
		}

		sessionState.todayNotePath = content.path;
		sessionState.todayNoteContent = content;
		return content;
	} catch (error) {
		toast.error(get(t)("notes.error.not_found"));
		console.error(error);
		return null;
	}
};
