/**
 * Simplified note data structure with markdown content.
 */
export type FormattedNote = {
	filename: string;
	formattedName: string;
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
	content: string; // Full markdown content
	metadata: NoteMetadata;
	sections: NoteSection[]; // Auto-detected headings
	cursorPosition?: number | null;
};
