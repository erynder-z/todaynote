import type { Editor } from "@milkdown/kit/core";
import type { MilkdownPlugin } from "@milkdown/kit/ctx";
import type { Snippet } from "svelte";
import type { EditorStore } from "#lib/stores/editor.svelte.ts";
import type { SelectionFloatingAnchorInstance } from "#lib/types/other.ts";
import type { EditorService } from "#lib/utils/editor.ts";
import type { NoteContentResponse } from "./notes.ts";

export interface EditorShortcutDeps {
	editor: EditorStore;
	getService: () => EditorService | null;
	getInstance: () => Editor | null;
}

export interface EditorViewProps {
	noteContent: NoteContentResponse | null;
	notePath: string | null;
}

export interface NoteEditorProps {
	noteContent: NoteContentResponse | null;
	notePath: string | null;
	editor: EditorStore;
}

export interface MilkdownEditorProps {
	content: string;
	onUpdate?: (markdown: string) => void;
	plugins?: MilkdownPlugin[];
	readonly?: boolean;
	activeThreadIndex?: number;
	onReady?: (instance: Editor) => void;
}

export interface ImageSizeMenuProps {
	container: HTMLDivElement | null;
	instance: Editor | null;
}

export interface FloatingToolbarProps {
	editorInstance: Editor | null;
}

export interface SelectionFloatingAnchorProps {
	children: Snippet<[boolean]>;
	editorWrapperClass?: string;
	linkInputActive?: boolean;
	editorInstance?: Editor | null;
}

export interface NoteFormatterProps {
	editorInstance: Editor | null;
	onLinkInputActive?: (active: boolean) => void;
	anchor?: SelectionFloatingAnchorInstance | null;
	visible?: boolean;
}
