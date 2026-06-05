export interface FormattedNote {
	filename: string;
	formattedName: string;
	preview: string;
	tags: string[];
	threads: string[];
	wordCount: number;
	hasCode: boolean;
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
	path: string;
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

export interface AppStatistics {
	totalNotes: number;
	totalTags: number;
	totalThreads: number;
	totalCharacters: number;
	totalWords: number;
	currentStreak: number;
	bestStreak: number;
	topTags: TagStat[];
	topThreads: ThreadStat[];
	dailyStats: DailyStat[];
	weekdayDistribution: number[];
	insights: InsightResponse[];
}

export interface InsightResponse {
	key: string;
	params: Record<string, string>;
}

export interface TagStat {
	name: string;
	count: number;
}

export interface ThreadStat {
	name: string;
	count: number;
}

export interface DailyStat {
	date: string;
	characterCount: number;
	wordCount: number;
}
