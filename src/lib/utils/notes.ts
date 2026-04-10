import { invoke } from "@tauri-apps/api/core";
import type { NoteContentResponse } from "$lib/types/notes";
import { MarkdownRenderCache } from "./renderCache";

const renderCache = new MarkdownRenderCache<string, string>(500);

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
export const jumpToSection = async (name: string, currentContent: string) => {
	try {
		const content = (await invoke("jump_to_section", {
			name,
			currentContent,
		})) as NoteContentResponse;
		return content;
	} catch (error) {
		console.error(`Error jumping to section ${name}:`, error);
		return null;
	}
};

/**
 * Detects top-level headings in markdown content and returns them as sections.
 * Only matches `# ` (single #), not `##` or `###`.
 */
export const detectSections = (
	content: string,
): Array<{
	name: string;
	level: number;
	startLine: number;
	endLine: number;
}> => {
	const lines = content.split("\n");
	const sections: Array<{
		name: string;
		level: number;
		startLine: number;
		endLine: number;
	}> = [];

	for (let i = 0; i < lines.length; i++) {
		// Only match top-level headings (# followed by space, not ##)
		if (lines[i]?.startsWith("# ") && !lines[i].startsWith("##")) {
			const name = lines[i].slice(2).trim();
			if (name) {
				// Update previous section's endLine
				if (sections.length > 0) sections[sections.length - 1].endLine = i;

				sections.push({
					level: 1,
					name,
					startLine: i,
					endLine: lines.length,
				});
			}
		}
	}

	return sections;
};
