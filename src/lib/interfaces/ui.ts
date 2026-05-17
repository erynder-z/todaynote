import type { ListNavigator } from "$lib/stores/listNav.svelte";
import type { ShortcutAction } from "../types/input";
import type { ToastType } from "../types/ui";
import type {
	SearchResult,
	TagSearchResult,
	ThreadSearchResult,
} from "./notes";

export interface SearchLayoutToolbarProps {
	onLayoutChange: (layout: "list" | "masonry") => void;
}

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

export interface SearchStatusViewProps {
	isSearching: boolean;
	hasResults: boolean;
	query: string;
	selectedTag: string | null;
}

export interface ShortcutHint {
	label: string;
	key?: string;
	action?: ShortcutAction;
	primary?: boolean;
	secondary?: boolean;
}

export interface Toast {
	id: string;
	message: string;
	type: ToastType;
	duration?: number;
}
