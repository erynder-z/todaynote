import { invoke } from "@tauri-apps/api/core";
import type { NoteContentResponse, NoteLineData } from "$lib/types/notes";
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
	if (!markdown?.trim()) return "&nbsp;";

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
 * Asks the backend to find or create a section and returns the updated note.
 */
export const jumpToSection = async (name: string) => {
	try {
		const content = (await invoke("jump_to_section", {
			name,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error(`Error jumping to section ${name}:`, error);
		return null;
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
 * Executes a slash command (e.g., "/Something") by clearing the line at the index
 * and jumping to the specified section. Returns the updated note content.
 */
export const executeSlashCommand = async (index: number, text: string) => {
	if (!text.trim().startsWith("/")) return null;

	const command = text.trim().slice(1).trim();
	if (!command) return null;

	try {
		await updateNoteLine(index, "");

		const updated = await jumpToSection(command);
		return updated;
	} catch (error) {
		console.error(`Error executing slash command "${command}":`, error);
		return null;
	}
};

/**
 * Transforms raw note content into the NoteLineData format for the editor.
 * This includes mapping headers to their corresponding keyboard shortcuts.
 */
export const mapNoteToEditorLines = (
	note: NoteContentResponse | null,
	primary: string,
	secondary: string,
): NoteLineData[] => {
	if (!note) return [];

	return note.lines.map((lineContent, i) => {
		const section = note.sections.find((s) => s.startLine === i);
		let shortcut = "";
		if (section) {
			const sectionIdx = note.sections.indexOf(section);
			if (sectionIdx !== undefined && sectionIdx < 9) {
				shortcut = `${secondary}${primary}${sectionIdx + 1}`;
			}
		}

		return {
			markdown: lineContent,
			html: "",
			sectionShortcut: shortcut,
		};
	});
};
