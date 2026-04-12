import { untrack } from "svelte";
import type { NoteContentResponse, NoteSection } from "$lib/types/notes";
import {
	detectSections,
	jumpToSection,
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
	private hasChanges = $state<boolean>(false);
	private autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;
	cursorPosition = $state<number | null>(null);
	sections = $state<NoteSection[]>([]);

	// Callback for section jumps
	onJump: (updated: NoteContentResponse) => void = () => {};

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
			this.refreshSections();
		} else if (noteContent?.content && currentContent !== noteContent.content) {
			// External content change (e.g., tag update) - sync content
			this.content = noteContent.content;
			this.hasChanges = false;
			this.sections = noteContent.sections ?? [];
		}
	}

	// --- Content Management ---

	/**
	 * Updates the markdown content and marks it as changed for auto-saving.
	 */
	updateContent(markdown: string) {
		this.content = markdown;
		this.hasChanges = true;
		this.scheduleAutoSave();
	}

	/**
	 * Triggers backend section detection. Call this when Enter is pressed
	 * (since new sections require a new line).
	 */
	async onEnterPressed() {
		this.refreshSections();
	}

	private refreshSections() {
		const content = this.content;
		detectSections(content).then((sections) => {
			this.sections = sections;
		});
	}

	/**
	 * Jumps to a section by name via the backend and updates cursor position.
	 */
	async jumpToSection(name: string) {
		const updated = await jumpToSection(name, this.content);
		if (updated) {
			this.content = updated.content;
			this.sections = updated.sections;

			// Calculate cursor position: end of last non-empty line in this section
			const section = updated.sections.find((s) => s.name === name);
			if (section) {
				const lines = updated.content.split("\n");
				// Walk backwards from endLine-1 to find the last non-empty line
				let targetLine = section.endLine - 1;
				while (targetLine > 0 && !(lines[targetLine] ?? "").trim())
					targetLine--;

				// Sum lengths of all lines up to and including targetLine, plus newlines
				let charPos = 0;
				for (let i = 0; i <= targetLine; i++)
					charPos += (lines[i]?.length || 0) + 1;

				this.cursorPosition = charPos;
			}

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
