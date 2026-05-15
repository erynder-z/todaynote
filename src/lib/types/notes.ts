export type FormattedNote = {
	filename: string;
	formattedName: string;
	preview: string;
	tags: string[];
	threads: string[];
	wordCount: number;
};

export type NoteListResponse = {
	notes: FormattedNote[];
	totalCount: number;
};

export type NoteMetadata = {
	formattedDate: string;
	tags: string[];
	raw: Record<string, string>;
};

export type NoteThread = {
	name: string;
	level: number;
	startLine: number;
	endLine: number;
	shortcut?: string;
};

export type NoteContentResponse = {
	content: string;
	metadata: NoteMetadata;
	threads: NoteThread[];
};

export type SearchResult = {
	filename: string;
	formattedName: string;
	excerpt: string;
	lineNumber: number;
	score: number;
	indices: number[];
};

export type ThreadSearchResult = {
	name: string;
	noteCount: number;
};

export type ThreadAggregationResult = {
	threadName: string;
	items: ThreadAggregationItem[];
};

export type ThreadAggregationItem = {
	filename: string;
	formattedDate: string;
	content: string;
};
