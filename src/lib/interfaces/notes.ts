export interface FormattedNote {
	filename: string;
	formattedName: string;
	preview: string;
	tags: string[];
	threads: string[];
	wordCount: number;
}

export interface NoteListResponse {
	notes: FormattedNote[];
	totalCount: number;
}

export interface NoteMetadata {
	formattedDate: string;
	tags: string[];
	raw: Record<string, string>;
}

export interface NoteThread {
	name: string;
	level: number;
	startLine: number;
	endLine: number;
	shortcut?: string;
}

export interface NoteContentResponse {
	content: string;
	metadata: NoteMetadata;
	threads: NoteThread[];
}

export interface SearchResult {
	filename: string;
	formattedName: string;
	excerpt: string;
	lineNumber: number;
	score: number;
	indices: number[];
}

export interface ThreadSearchResult {
	name: string;
	noteCount: number;
}

export interface TagSearchResult {
	name: string;
	noteCount: number;
}

export interface ThreadAggregationResult {
	threadName: string;
	items: ThreadAggregationItem[];
}

export interface ThreadAggregationItem {
	filename: string;
	formattedDate: string;
	content: string;
}
