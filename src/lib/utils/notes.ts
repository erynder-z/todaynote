import { invoke } from "@tauri-apps/api/core";
import type {
	NoteContentResponse,
	NoteThread,
	SearchResult,
	TagSearchResult,
	ThreadAggregationResult,
	ThreadSearchResult,
} from "$lib/interfaces/notes";
import { processSearchResults } from "./search";

/**
 * Service class for all note-related operations including search, reading, saving,
 * thread management, and tag operations.
 */
export class NotesService {
	/**
	 * Performs a full-text search across all notes.
	 */
	async searchNotes(
		query: string,
		isFuzzy: boolean,
		options?: {
			minScore?: number;
			maxResults?: number;
			filenameFilter?: string;
			sortBy?: string;
		},
	): Promise<SearchResult[]> {
		try {
			const results = (await invoke("search_notes", {
				query,
				isFuzzy,
			})) as SearchResult[];

			if (
				options &&
				(options.minScore !== undefined ||
					options.maxResults !== undefined ||
					options.filenameFilter ||
					options.sortBy)
			)
				return await processSearchResults(results, options);

			return results;
		} catch (error) {
			console.error("Error searching notes:", error);
			return [];
		}
	}

	/**
	 * Searches for unique thread names across all notes.
	 */
	async searchThreads(
		query: string,
		isFuzzy: boolean,
	): Promise<ThreadSearchResult[]> {
		try {
			const results = (await invoke("search_threads", {
				query,
				isFuzzy,
			})) as ThreadSearchResult[];
			return results;
		} catch (error) {
			console.error("Error searching threads:", error);
			return [];
		}
	}

	/**
	 * Searches for unique tags across all notes.
	 */
	async searchTags(
		query: string,
		isFuzzy: boolean,
	): Promise<TagSearchResult[]> {
		try {
			const results = (await invoke("search_tags", {
				query,
				isFuzzy,
			})) as TagSearchResult[];
			return results;
		} catch (error) {
			console.error("Error searching tags:", error);
			return [];
		}
	}

	/**
	 * Finds all notes that contain a specific tag.
	 */
	async searchNotesByTag(
		tag: string,
		query = "",
		isFuzzy = false,
	): Promise<SearchResult[]> {
		try {
			const results = (await invoke("search_notes_by_tag", {
				tag,
				query,
				isFuzzy,
			})) as SearchResult[];
			return results;
		} catch (error) {
			console.error(`Error searching notes by tag ${tag}:`, error);
			return [];
		}
	}

	/**
	 * Aggregates content from all threads with the given thread name.
	 */
	async aggregateThread(name: string): Promise<ThreadAggregationResult | null> {
		try {
			const result = (await invoke("aggregate_thread", {
				name,
			})) as ThreadAggregationResult;
			return result;
		} catch (error) {
			console.error(`Error aggregating thread ${name}:`, error);
			return null;
		}
	}

	/**
	 * Reads the full markdown content of a note file from the given path.
	 */
	async readNoteContent(path: string): Promise<NoteContentResponse | null> {
		try {
			const content = (await invoke("read_note_content", {
				path,
			})) as NoteContentResponse;
			return content;
		} catch (error) {
			console.error(`Error reading note content from ${path}:`, error);
			return null;
		}
	}

	/**
	 * Saves the entire markdown content of a note to the specified path.
	 */
	async saveNoteContent(
		path: string,
		content: string,
	): Promise<NoteContentResponse | null> {
		try {
			const result = (await invoke("save_note_content", {
				path,
				content,
			})) as NoteContentResponse;
			return result;
		} catch (error) {
			console.error(`Error saving note content to ${path}:`, error);
			return null;
		}
	}

	/**
	 * Adds a tag to the current note and returns the updated note content.
	 */
	async addNoteTag(
		tag: string,
		currentContent: string,
	): Promise<NoteContentResponse | undefined> {
		try {
			const content = (await invoke("add_note_tag", {
				tag,
				currentContent,
			})) as NoteContentResponse;
			return content;
		} catch (error) {
			console.error("Error adding note tag:", error);
		}
	}

	/**
	 * Removes a tag from the current note and returns the updated note content.
	 */
	async removeNoteTag(
		tag: string,
		currentContent: string,
	): Promise<NoteContentResponse | undefined> {
		try {
			const content = (await invoke("remove_note_tag", {
				tag,
				currentContent,
			})) as NoteContentResponse;
			return content;
		} catch (error) {
			console.error("Error removing note tag:", error);
		}
	}

	/**
	 * Retrieves tag suggestions based on a search query.
	 */
	async getTagSuggestions(query: string): Promise<string[]> {
		try {
			const tags = (await invoke("get_tag_suggestions", { query })) as string[];
			return tags;
		} catch (error) {
			console.error("Error getting tag suggestions:", error);
			return [];
		}
	}

	/**
	 * Asks the backend to find or create a thread and returns the updated note.
	 * Pass the current content to ensure unsaved edits are not lost.
	 */
	async ensureThread(
		name: string,
		currentContent: string,
	): Promise<NoteContentResponse | null> {
		try {
			const content = (await invoke("ensure_thread", {
				name,
				currentContent,
			})) as NoteContentResponse;
			return content;
		} catch (error) {
			console.error(`Error ensuring thread ${name}:`, error);
			return null;
		}
	}

	/**
	 * Asks the backend to detect threads in the given markdown content.
	 * If path is provided, the backend will load the full content from disk to access frontmatter.
	 */
	async detectThreads(content: string, path?: string): Promise<NoteThread[]> {
		try {
			return (await invoke("detect_threads", { content, path })) as NoteThread[];
		} catch (error) {
			console.error("Error detecting threads:", error);
			return [];
		}
	}

	/**
	 * Removes a thread by name from the current note and returns the updated note content.
	 * Pass the current content to ensure unsaved edits are not lost.
	 */
	async removeThread(
		threadId: string,
		currentContent: string,
	): Promise<NoteContentResponse | null> {
		try {
			const content = (await invoke("remove_thread", {
				threadId,
				currentContent,
			})) as NoteContentResponse;
			return content;
		} catch (error) {
			console.error(`Error removing thread ${threadId}:`, error);
			return null;
		}
	}

	/**
	 * Formats a note's filename into a human-readable, localized string.
	 */
	formatNoteName(filename: string, currentLocale: string): string {
		const withoutExt = filename.replace(/.md$/, "");

		const datePattern = /^(d{4})-(d{2})-(d{2})$/;
		const match = withoutExt.match(datePattern);
		if (!match) return withoutExt;

		const year = Number.parseInt(match[1], 10);
		const month = Number.parseInt(match[2], 10) - 1;
		const day = Number.parseInt(match[3], 10);
		const dateObj = new Date(year, month, day);

		return dateObj.toLocaleDateString(currentLocale, {
			weekday: "long",
			year: "numeric",
			month: "long",
			day: "numeric",
		});
	}
}

/**
 * Singleton instance of the NotesService for convenience.
 */
export const notesService = new NotesService();
