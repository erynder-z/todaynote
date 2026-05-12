export type FormattedNote = {
	filename: string;
	formattedName: string;
	preview: string;
	tags: string[];
	sections: string[];
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

export type NoteSection = {
	name: string;
	level: number;
	startLine: number;
	endLine: number;
	shortcut?: string;
};

export type NoteContentResponse = {
	content: string;
	metadata: NoteMetadata;
	sections: NoteSection[];
};

export type SearchResult = {
	filename: string;
	formattedName: string;
	excerpt: string;
	lineNumber: number;
	score: number;
	indices: number[];
};
