import type { SessionState } from "$lib/interfaces/appState";
/**
 * Global application state including current note information and UI state.
 */
export const sessionState = $state<SessionState>({
	todayNotePath: null,
	todayNoteContent: null,
	aggregatedThread: null,
	activePopup: null,
	isMac: false,
	sidebarOpen: typeof window !== "undefined" ? window.innerWidth > 1024 : false,
	selectedThreadForOptions: null,
	threadShortcutsMode: "navigation",
});
