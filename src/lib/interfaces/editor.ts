import type { Editor } from "@milkdown/kit/core";
import type { EditorStore } from "#lib/stores/editor.svelte.js";
import type { EditorService } from "#lib/utils/editor.js";

export interface EditorShortcutDeps {
	editor: EditorStore;
	getService: () => EditorService | null;
	getInstance: () => Editor | null;
}
