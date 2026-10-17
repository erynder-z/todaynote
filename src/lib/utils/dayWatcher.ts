import { invoke } from "@tauri-apps/api/core";
import type { DayBoundaryStatus } from "#lib/interfaces/date.ts";
import type { NoteContentResponse } from "#lib/interfaces/notes.ts";
import { sessionState } from "../stores/sessionState.svelte";
import { toast } from "../stores/toast.svelte";

/**
 * Opens today's daily note by calling the backend command which
 * creates the note if needed and returns its content.
 */
export const openTodayNote = async () => {
	try {
		const content = (await invoke("open_todays_note")) as NoteContentResponse;

		sessionState.todayNotePath = content.path;
		sessionState.todayNoteContent = content;
		sessionState.pendingThreadJump = null;
		return content;
	} catch (error) {
		toast.error("notes.error.load");
		console.error("Failed to open today's note:", error);
		return null;
	}
};

/**
 * Monitors for day boundary crossings (midnight) while the app is running.
 *
 * Uses two complementary triggers:
 * 1. A periodic interval that checks every 60 seconds.
 * 2. Window focus / visibility listeners to catch sleep-wake transitions instantly.
 *
 * When a new day is detected, shows a persistent toast with an action button
 * to switch to today's note. Only prompts once per new date.
 */
export class DayWatcher {
	private intervalId: ReturnType<typeof setInterval> | null = null;
	private lastPromptedDate: string | null = null;
	private boundHandleFocus: () => void;
	private boundHandleVisibility: () => void;

	constructor() {
		this.boundHandleFocus = () => this.check();
		this.boundHandleVisibility = () => {
			if (document.visibilityState === "visible") this.check();
		};
	}

	/**
	 * Starts watching for day boundary changes.
	 */
	start() {
		const checkInterval = 60_000; // 60 seconds
		this.intervalId = setInterval(() => this.check(), checkInterval);
		window.addEventListener("focus", this.boundHandleFocus);
		document.addEventListener("visibilitychange", this.boundHandleVisibility);
	}

	/**
	 * Stops all watchers and cleans up listeners.
	 */
	stop() {
		if (this.intervalId) {
			clearInterval(this.intervalId);
			this.intervalId = null;
		}
		window.removeEventListener("focus", this.boundHandleFocus);
		document.removeEventListener(
			"visibilitychange",
			this.boundHandleVisibility,
		);
	}

	/**
	 * Performs the day boundary check by delegating to the Rust backend.
	 */
	private async check() {
		try {
			const status = (await invoke("check_day_boundary")) as DayBoundaryStatus;

			if (status.isNewDay && status.currentDate !== this.lastPromptedDate) {
				this.lastPromptedDate = status.currentDate;
				this.promptNewDay();
			}
		} catch (error) {
			console.error("Day boundary check failed:", error);
		}
	}

	/**
	 * Shows a persistent toast prompting the user to switch to today's note.
	 */
	private promptNewDay() {
		toast.showWithAction(
			"notes.new_day_prompt",
			{
				label: "notes.switch_to_today",
				onClick: () => openTodayNote(),
			},
			"info",
			0,
		);
	}
}

export const dayWatcher = new DayWatcher();
