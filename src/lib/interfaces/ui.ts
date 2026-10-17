import type { Snippet } from "svelte";
import type { ListNavigator } from "#lib/stores/listNav.svelte.ts";
import type { ShortcutAction } from "../types/input.ts";
import type { ModalSize, ToastType } from "../types/ui.ts";
import type {
	SearchResult,
	TagSearchResult,
	ThreadSearchResult,
} from "./notes";

export interface LayoutToggleProps {
	onLayoutChange: (layout: "list" | "masonry") => void;
}

export interface SearchInputProps {
	query: string;
	isFuzzy: boolean;
	searchMode: string;
	selectedTag: string | null;
	onInput: () => void;
	onClearTag: () => void;
	onClearQuery: () => void;
	onToggleFuzzy?: () => void;
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

export interface IdentIconProps {
	title?: string;
	size?: number;
	style?: "dotmatrix" | "round" | "none";
}

export interface IdentIconVariantProps {
	title?: string;
	size?: string; // relative units
}

export interface ModalProps {
	title?: string;
	children: Snippet;
	size?: ModalSize;
	showLayoutToggle?: boolean;
}

export interface SearchSidebarProps {
	searchMode: "notes" | "threads" | "tags";
	selectedTag: string | null;
	onModeChange: (mode: "notes" | "threads" | "tags") => void;
}

export interface ToastProps {
	t: Toast;
}

export interface ShortcutHint {
	label: string;
	key?: string;
	action?: ShortcutAction;
	primary?: boolean;
	secondary?: boolean;
}

export interface ToastAction {
	label: string;
	onClick: () => void;
}

export interface Toast {
	id: string;
	message: string;
	type: ToastType;
	duration?: number;
	action?: ToastAction;
}
