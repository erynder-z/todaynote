import type { SessionState } from "$lib/types/appState";

/**
 * Global application state including current note information and UI state.
 */
export const sessionState = $state<SessionState>({
	todayNotePath: null,
	todayNoteContent: null,
	aggregatedThread: null,
	activePopup: null,
	isMac: false,
	sidebarOpen: false,
});
