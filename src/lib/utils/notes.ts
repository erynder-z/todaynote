import { invoke } from "@tauri-apps/api/core";
import type { NoteContentResponse } from "$lib/types/notes";

/**
 * Reads the full markdown content of a note file from the given path.
 */
export const readNoteContent = async (path: string) => {
	try {
		const content = (await invoke("read_note_content", {
			path,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error(`Error reading note content from ${path}:`, error);
		return null;
	}
};

/**
 * Saves the entire markdown content of a note to the specified path.
 */
export const saveNoteContent = async (path: string, content: string) => {
	try {
		await invoke("save_note_content", { path, content });
		return true;
	} catch (error) {
		console.error(`Error saving note content to ${path}:`, error);
		return false;
	}
};

/**
 * Adds a tag to the current note and returns the updated note content.
 */
export const addNoteTag = async (tag: string) => {
	try {
		const content = (await invoke("add_note_tag", {
			tag,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error("Error adding note tag:", error);
	}
};

/**
 * Removes a tag from the current note and returns the updated note content.
 */
export const removeNoteTag = async (tag: string) => {
	try {
		const content = (await invoke("remove_note_tag", {
			tag,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error("Error removing note tag:", error);
	}
};

/**
 * Retrieves all tags from all notes, sorted by frequency.
 */
export const getAllTags = async () => {
	try {
		const tags = (await invoke("get_all_tags")) as string[];
		return tags;
	} catch (error) {
		console.error("Error getting all tags:", error);
		return [];
	}
};

/**
 * Retrieves tag suggestions based on a search query.
 */
export const getTagSuggestions = async (query: string) => {
	try {
		const tags = (await invoke("get_tag_suggestions", { query })) as string[];
		return tags;
	} catch (error) {
		console.error("Error getting tag suggestions:", error);
		return [];
	}
};

/**
 * Asks the backend to find or create a section and returns the updated note.
 * Pass the current content to ensure unsaved edits are not lost.
 */
export const ensureSection = async (name: string, currentContent: string) => {
	try {
		const content = (await invoke("ensure_section", {
			name,
			currentContent,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error(`Error ensuring section ${name}:`, error);
		return null;
	}
};

/**
 * Asks the backend to detect sections in the given markdown content.
 */
export const detectSections = async (
	content: string,
): Promise<
	Array<{
		name: string;
		level: number;
		startLine: number;
		endLine: number;
	}>
> => {
	try {
		return (await invoke("detect_sections", { content })) as Array<{
			name: string;
			level: number;
			startLine: number;
			endLine: number;
		}>;
	} catch (error) {
		console.error("Error detecting sections:", error);
		return [];
	}
};
