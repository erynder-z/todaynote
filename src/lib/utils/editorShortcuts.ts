import { tick } from "svelte";
import type { EditorShortcutDeps } from "#lib/interfaces/editor.js";
import type { NoteThread } from "#lib/interfaces/notes.js";
import type { ShortcutAction, ShortcutCallback } from "#lib/types/input.js";
import { tagSuggestionShortcuts } from "../config/shortcuts";
import { sessionState } from "../stores/sessionState.svelte";
import { navigateToLastAvailable, navigateToOffset } from "./dailyNote";

/**
 * Builds the keyboard shortcut handlers for the note editor.
 *
 * Thread navigation (jump by id / by index) is shared between the
 * `jumpByNumber` shortcut and external callers, so it's exposed as a
 * method on the returned object as well.
 */
export const createEditorShortcuts = (
	deps: EditorShortcutDeps,
): {
	actions: Partial<Record<ShortcutAction, ShortcutCallback>>;
	jumpToThread: (threadId: string) => Promise<void>;
} => {
	const { editor, getService, getInstance } = deps;

	/** Jump to a thread by id, retrying once after a tick if not found yet. */
	const jumpToThread = async (threadId: string) => {
		if (!getInstance()) return;

		const tryJump = () => {
			const service = getService();
			if (!service) return true;
			const threadIndex = editor.threads.findIndex(
				(nt: NoteThread) => nt.id === threadId,
			);
			if (threadIndex !== -1) {
				service.jumpToThreadByIndex(threadIndex);
				return true;
			}
			return false;
		};

		if (tryJump()) return;
		await tick();
		tryJump();
	};

	/**
	 * When in Navigation Mode: jumps to a thread based on its index.
	 * When in Thread Options Mode: opens the thread options popup.
	 */
	const jumpToThreadByIndex = async (idx: number) => {
		const thread = editor.threads[idx];
		if (!thread?.id) return;

		if (sessionState.threadShortcutsMode === "navigation") {
			await jumpToThread(thread.id);
		} else {
			sessionState.selectedThreadForOptions = thread;
			sessionState.activePopup = "threadOptions";
		}
	};

	/**
	 * Focuses the end of the editor content.
	 */
	const handleFocusLastLine = (): boolean => {
		if (sessionState.activePopup !== null) return false;
		if (getInstance() && getService()) {
			getService()?.focusEnd();
			return true;
		}
		return false;
	};

	/**
	 * Handles numeric shortcuts for thread navigation or options.
	 */
	const handleJumpByNumber = (e: KeyboardEvent): boolean => {
		if (
			sessionState.activePopup !== null &&
			sessionState.activePopup !== "threadOptions"
		)
			return false;

		const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
		if (idx !== -1 && idx < editor.threads.length) {
			jumpToThreadByIndex(idx);
			return true;
		}
		return false;
	};

	/**
	 * Navigates to the previous day's note.
	 */
	const handleNavigateYesterday = async (e: Event) => {
		if (sessionState.activePopup !== null) {
			e.preventDefault();
			return;
		}
		await navigateToOffset(-1);
	};

	/**
	 * Navigates to the most recent available note.
	 */
	const handleNavigateLastAvailable = async (e: Event) => {
		if (sessionState.activePopup !== null) {
			e.preventDefault();
			return;
		}
		await navigateToLastAvailable();
	};

	/**
	 * Navigates to today's note.
	 */
	const handleNavigateToday = async (e: Event) => {
		if (sessionState.activePopup !== null) {
			e.preventDefault();
			return;
		}
		await navigateToOffset(0);
	};

	return {
		actions: {
			focusLastLine: handleFocusLastLine,
			jumpByNumber: handleJumpByNumber,
			navigateYesterday: handleNavigateYesterday,
			navigateLastAvailable: handleNavigateLastAvailable,
			navigateToday: handleNavigateToday,
		},
		jumpToThread,
	};
};
