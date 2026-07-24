import { onMount } from "svelte";
import { inputManager } from "$lib/stores/input.svelte";
import type { ShortcutAction, ShortcutCallback } from "$lib/types/input";

/**
 * Shortcut registration helper that handles cleanup automatically.
 */
export const useShortcuts = (
	scopeOrActions: string | Partial<Record<ShortcutAction, ShortcutCallback>>,
	actionsOrAutoActivate?:
		| Partial<Record<ShortcutAction, ShortcutCallback>>
		| boolean,
	autoActivate: boolean = true,
) => {
	onMount(() => {
		// Parse arguments based on type
		let scope: string = "global";
		let actions: Partial<Record<ShortcutAction, ShortcutCallback>> = {};

		if (typeof scopeOrActions === "string") {
			scope = scopeOrActions;
			if (typeof actionsOrAutoActivate === "object") {
				actions = actionsOrAutoActivate;
			} else if (typeof actionsOrAutoActivate === "boolean") {
				autoActivate = actionsOrAutoActivate;
			}
		} else {
			// First argument is actions object (backward compatibility)
			actions = scopeOrActions;
		}

		// Handle global vs scoped shortcuts
		if (scope === "global") {
			return inputManager.registerActions(actions);
		} else {
			const unregister = inputManager.registerScopedActions(scope, actions);

			if (autoActivate) inputManager.activateScope(scope);

			return () => {
				unregister();
				if (autoActivate) inputManager.deactivateScope(scope);
			};
		}
	});
};
