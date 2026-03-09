import { invoke } from "@tauri-apps/api/core";
import type { NoteContentResponse } from "$lib/types/notes";
import { MarkdownRenderCache } from "./renderCache";

const renderCache = new MarkdownRenderCache<string, string>(500);

/**
 * Reads the full text content of a note file from the given path.
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
 * Renders a markdown string to HTML, utilizing a client-side cache for performance.
 */
export const renderMarkdown = async (markdown: string) => {
	if (!markdown || !markdown.trim()) return "&nbsp;";

	// Return from cache if string is in cache
	const cached = renderCache.get(markdown);
	if (cached !== undefined) return cached;

	try {
		const html = (await invoke("render_markdown", { markdown })) as string;

		// Store the new result in cache
		renderCache.set(markdown, html);
		return html;
	} catch (error) {
		console.error("Error rendering markdown:", error);
		return markdown;
	}
};

/**
 * Saves the entire content of a note to the specified path.
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
 * Updates a specific line of the currently active note in the backend.
 */
export const updateNoteLine = async (index: number, content: string) => {
	try {
		await invoke("update_note_line", { index, content });
		return true;
	} catch (error) {
		console.error("Error updating note line:", error);
		return false;
	}
};

/**
 * Inserts a new line into the currently active note at the specified index.
 */
export const insertNoteLine = async (index: number, content: string) => {
	try {
		await invoke("insert_note_line", { index, content });
		return true;
	} catch (error) {
		console.error("Error inserting note line:", error);
		return false;
	}
};

/**
 * Removes a line from the currently active note at the specified index.
 */
export const deleteNoteLine = async (index: number) => {
	try {
		await invoke("delete_note_line", { index });
		return true;
	} catch (error) {
		console.error("Error deleting note line:", error);
		return false;
	}
};
