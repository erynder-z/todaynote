import type { ListNavigator } from "$lib/stores/listNav.svelte";
import type {
	SearchResult,
	TagSearchResult,
	ThreadSearchResult,
} from "$lib/types/notes";

export interface SearchResultsContainerProps {
	results: SearchResult[] | ThreadSearchResult[] | TagSearchResult[];
	searchMode: "notes" | "threads" | "tags";
	selectedTag: string | null;
	query: string;
	nav: ListNavigator;
	masonryLayout: { handleKey: (e: KeyboardEvent) => boolean } | null;
	onSelectNote: (result: SearchResult) => Promise<void>;
	onSelectThread: (thread: ThreadSearchResult) => Promise<void>;
	onSelectTag: (tag: TagSearchResult) => void;
}
