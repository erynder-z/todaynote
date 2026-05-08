/**
 * Simplified note data structure with markdown content.
 */
export type FormattedNote = {
	filename: string;
	formattedName: string;
	preview: string;
	tags: string[];
	sections: string[];
	wordCount: number;
};

export type NoteMetadata = {
	formattedDate: string;
	tags: string[];
	raw: Record<string, string>;
};

/**
 * A heading/section detected in the markdown content.
 */
export type NoteSection = {
	name: string;
	level: number; // Heading depth (1 for #, 2 for ##, etc.)
	startLine: number; // 0-based line index where heading starts
	endLine: number; // 0-based line index where section ends (exclusive)
	shortcut?: string; // e.g. "Alt+1" for quick navigation
};

/**
 * The complete note content response from the backend.
 */
export type NoteContentResponse = {
	content: string;
	metadata: NoteMetadata;
	sections: NoteSection[];
};

/**
 * A search match from the note archive.
 */
export type SearchResult = {
	filename: string;
	formattedName: string;
	excerpt: string;
	lineNumber: number;
	score: number;
	indices: number[];
};
