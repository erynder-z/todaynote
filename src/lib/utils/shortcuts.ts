import { onMount } from "svelte";
import { inputManager } from "$lib/stores/input.svelte";
import type { ShortcutAction, ShortcutCallback } from "$lib/types/input";

/**
 * Shortcut registration helper that handles cleanup automatically.
 */
export const useShortcuts = (
	actions: Partial<Record<ShortcutAction, ShortcutCallback>>,
) => {
	onMount(() => {
		return inputManager.registerActions(actions);
	});
};
