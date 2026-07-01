import { untrack } from "svelte";
import type { NoteContentResponse, NoteThread } from "$lib/interfaces/notes";
import { notesService } from "$lib/utils/notes";

/**
 * Manages the state and logic for the Note Editor.
 * Handles markdown content, auto-saving, and thread navigation.
 */
export class EditorStore {
	content = $state<string>("");
	noteContent = $state<NoteContentResponse | null>(null);
	notePath = $state<string | null>(null);
	hasChanges = $state<boolean>(false);
	pendingExternalUpdate = $state<boolean>(false);
	private autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;
	private threadRefreshTimeout: ReturnType<typeof setTimeout> | null = null;
	threads = $state<NoteThread[]>([]);

	// Callback for when content changes (saves, jumps, etc.) to sync back to the parent
	onContentUpdate: (updated: NoteContentResponse) => void = () => {};

	// Function to jump to a thread (set by NoteEditor component)
	jumpToThread: (name: string) => void = () => {};

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
			this.refreshThreads();
		} else if (
			noteContent?.content !== undefined &&
			currentContent !== noteContent.content
		) {
			// External content change (e.g., tag update) - sync content
			this.content = noteContent.content;
			this.pendingExternalUpdate = true;
			this.threads = noteContent.threads ?? [];
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
		this.scheduleThreadRefresh();
	}

	private scheduleThreadRefresh() {
		if (this.threadRefreshTimeout) clearTimeout(this.threadRefreshTimeout);

		this.threadRefreshTimeout = setTimeout(() => {
			untrack(() => this.refreshThreads());
		}, 250);
	}

	private refreshThreads() {
		const content = this.content;
		notesService.detectThreads(content).then((threads) => {
			this.threads = threads;
		});
	}

	/**
	 * Creates a thread via the backend and updates the store.
	 * Navigation logic remains in the component.
	 */
	async ensureThreadExists(name: string) {
		const updated = await notesService.ensureThread(name, this.content);
		if (updated) {
			this.content = updated.content;
			this.threads = updated.threads;
			this.pendingExternalUpdate = true;
			this.hasChanges = false;
			this.onContentUpdate(updated);
		}
	}

	/**
	 * Removes a thread from the current note and updates the store.
	 */
	async removeThread(name: string) {
		const updated = await notesService.removeThread(name, this.content);
		if (updated) {
			this.content = updated.content;
			this.threads = updated.threads;
			this.pendingExternalUpdate = true;
			this.hasChanges = false;
		}
	}

	// --- Persistence ---

	/**
	 * Persists the entire note content to the backend.
	 */
	async flush() {
		if (this.notePath && this.hasChanges) {
			const updated = await notesService.saveNoteContent(
				this.notePath,
				this.content,
			);
			if (updated) {
				this.threads = updated.threads;
				if (this.noteContent) {
					this.noteContent.metadata = updated.metadata;
					this.noteContent.threads = updated.threads;
					this.noteContent.path = updated.path;
				}
			}
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
