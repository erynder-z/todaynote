export interface FormattedNote {
	filename: string;
	formattedName: string;
	created: string | null;
	noteType: string;
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
	id: string;
	name: string;
	level?: number;
	startLine: number;
	endLine: number;
	shortcut?: string;
	pinned?: boolean;
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
	created: string | null;
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
	created: string | null;
	content: string;
	threadId: string;
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

export interface AggregatedThreadItem {
	filename: string;
	threadId: string;
	content: string;
	created: string | null;
}

export interface PinnedThreadItem {
	threadId: string;
	threadName: string;
	filename: string;
	formattedDate: string;
	excerpt: string;
	lineNumber: number;
}
