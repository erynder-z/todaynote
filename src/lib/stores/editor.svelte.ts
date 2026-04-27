import { untrack } from "svelte";
import type { NoteContentResponse, NoteSection } from "$lib/types/notes";
import {
	detectSections,
	ensureSection,
	saveNoteContent,
} from "$lib/utils/notes";

/**
 * Manages the state and logic for the Note Editor.
 * Handles markdown content, auto-saving, and section navigation.
 */
export class EditorStore {
	content = $state<string>("");
	noteContent = $state<NoteContentResponse | null>(null);
	notePath = $state<string | null>(null);
	hasChanges = $state<boolean>(false);
	pendingExternalUpdate = $state<boolean>(false);
	private autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;
	sections = $state<NoteSection[]>([]);

	// Callback for section jumps
	onJump: (updated: NoteContentResponse) => void = () => {};

	// Function to jump to a section (set by NoteEditor component)
	jumpToSection: (name: string) => void = () => {};

	// --- Initialization ---

	/**
	 * Synchronizes the store with the current note props.
	 */
	sync(noteContent: NoteContentResponse | null, notePath: string | null) {
		const pathChanged = untrack(() => this.notePath) !== notePath;
		const currentContent = untrack(() => this.content);

		this.notePath = notePath;
		this.noteContent = noteContent;

		if (pathChanged) {
			this.content = noteContent?.content ?? "";
			this.hasChanges = false;
			this.pendingExternalUpdate = true;
			this.refreshSections();
		} else if (
			noteContent?.content !== undefined &&
			currentContent !== noteContent.content
		) {
			// External content change (e.g., tag update) - sync content
			this.content = noteContent.content;
			this.pendingExternalUpdate = true;
			this.sections = noteContent.sections ?? [];
			this.hasChanges = false;
		}
	}

	// --- Content Management ---

	/**
	 * Updates the markdown content and marks it as changed for auto-saving.
	 */
	updateContent(markdown: string) {
		this.content = markdown;

		if (this.noteContent) this.noteContent.content = markdown;

		this.hasChanges = true;
		this.scheduleAutoSave();
		this.refreshSections();
	}

	private refreshSections() {
		const content = this.content;
		detectSections(content).then((sections) => {
			this.sections = sections;
		});
	}

	/**
	 * Creates a section via the backend and updates the store.
	 * Navigation logic remains in the component.
	 */
	async ensureSectionExists(name: string) {
		const updated = await ensureSection(name, this.content);
		if (updated) {
			this.content = updated.content;
			this.sections = updated.sections;
			this.pendingExternalUpdate = true;
			this.hasChanges = false;
			this.onJump(updated);
		}
	}

	// --- Persistence ---

	/**
	 * Persists the entire note content to the backend.
	 */
	async flush() {
		if (this.notePath && this.hasChanges) {
			await saveNoteContent(this.notePath, this.content);
			this.hasChanges = false;
		}
	}

	private scheduleAutoSave() {
		if (this.autoSaveTimeout) clearTimeout(this.autoSaveTimeout);

		this.autoSaveTimeout = setTimeout(() => {
			untrack(() => this.flush());
		}, 500);
	}
}
