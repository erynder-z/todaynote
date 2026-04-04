import { untrack } from "svelte";
import type { NoteContentResponse, NoteLineData } from "$lib/types/notes";
import {
	deleteNoteLine,
	executeSlashCommand,
	insertNoteLine,
	mapNoteToEditorLines,
	updateNoteLine,
} from "$lib/utils/notes";
import { inputManager } from "./input.svelte";

/**
 * Manages the state and logic for the Note Editor.
 * Handles line management, auto-saving, and keyboard navigation.
 */
export class EditorStore {
	lines = $state<NoteLineData[]>([]);
	activeIndex = $state<number | null>(null);
	noteContent = $state<NoteContentResponse | null>(null);
	notePath = $state<string | null>(null);
	private changedLineIndex = $state<number | null>(null);
	private autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;

	// --- Initialization ---

	/**
	 * Synchronizes the store with the current note props.
	 * Handles loading new notes and resetting state.
	 */
	sync(noteContent: NoteContentResponse | null, notePath: string | null) {
		const pathChanged = this.notePath !== notePath;
		const contentChanged = this.noteContent !== noteContent;

		this.noteContent = noteContent;
		this.notePath = notePath;

		if (pathChanged) {
			this.loadLines();
			this.focusLastLine();
			this.changedLineIndex = null;
		} else if (contentChanged) {
			this.loadLines();
		}
	}

	/**
	 * Parses raw note content into editor lines.
	 */
	loadLines() {
		this.lines = mapNoteToEditorLines(
			this.noteContent,
			inputManager.primaryLabel,
			inputManager.secondaryLabel,
		);
	}

	// --- Line Management ---

	/**
	 * Updates the content of a specific line and marks it as changed for auto-saving.
	 */
	updateLine(index: number, markdown: string) {
		if (this.lines[index]) {
			this.lines[index].markdown = markdown;
			this.changedLineIndex = index;
			this.scheduleAutoSave(index);
		}
	}

	/**
	 * Adds a new empty line after the specified index.
	 */
	async insertLine(i: number) {
		this.lines.splice(i + 1, 0, { markdown: "", html: "" });
		this.activeIndex = i + 1;
		await insertNoteLine(i + 1, "");
	}

	/**
	 * Deletes the line at the specified index.
	 */
	async deleteLine(i: number) {
		this.lines.splice(i, 1);
		this.activeIndex = Math.max(0, i - 1);
		await deleteNoteLine(i);
	}

	/**
	 * Appends an empty line at the end if needed and focuses it.
	 */
	focusLastLine() {
		const lastLine = this.lines[this.lines.length - 1];
		if (this.lines.length === 0 || lastLine?.markdown.trim() !== "") {
			this.lines.push({ markdown: "", html: "" });
			insertNoteLine(this.lines.length - 1, "");
		}
		this.activeIndex = this.lines.length - 1;
	}

	// --- Persistence ---

	/**
	 * Persists a specific line to the backend.
	 */
	async flush(index: number) {
		if (this.lines[index]) {
			const content = this.lines[index].markdown;
			if (this.changedLineIndex === index) this.changedLineIndex = null;
			await updateNoteLine(index, content);
		}
	}

	private scheduleAutoSave(index: number) {
		if (this.autoSaveTimeout) clearTimeout(this.autoSaveTimeout);

		this.autoSaveTimeout = setTimeout(() => {
			untrack(() => {
				if (this.changedLineIndex === index) this.flush(index);
			});
		}, 500);
	}

	/**
	 * Ensures unsaved changes are flushed when moving between lines.
	 */
	handleLineSwitch(newIndex: number | null) {
		untrack(() => {
			if (this.changedLineIndex !== null && this.changedLineIndex !== newIndex)
				this.flush(this.changedLineIndex);
		});
	}

	// --- Navigation & Commands ---

	/**
	 * Handles keyboard events for the editor.
	 */
	async handleKeyDown(e: KeyboardEvent, i: number): Promise<boolean> {
		if (e.key === "Enter") {
			const updated = await executeSlashCommand(i, this.lines[i].markdown);
			if (updated) {
				this.lines[i].markdown = "";
				this.onJump(updated);
				return true;
			}

			await this.insertLine(i);
			return true;
		}

		switch (e.key) {
			case "Backspace":
				if (this.lines[i].markdown === "" && this.lines.length > 1) {
					await this.deleteLine(i);
					return true;
				}
				break;
			case "ArrowUp":
				this.navigate(i, "up");
				return true;
			case "ArrowDown":
				this.navigate(i, "down");
				return true;
		}

		return false;
	}

	private navigate(i: number, direction: "up" | "down") {
		const nextIndex = direction === "up" ? i - 1 : i + 1;
		if (nextIndex >= 0 && nextIndex < this.lines.length)
			this.activeIndex = nextIndex;
	}

	/**
	 * Internal callback to update state after a jump or slash command.
	 * Overridden by the component to sync props.
	 */
	onJump: (updated: NoteContentResponse) => void = () => {};
}
