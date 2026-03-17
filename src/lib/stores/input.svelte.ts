/**
 * Centralized Input Manager for handling global keyboard shortcuts and tracking key states.
 */

import { defaultShortcuts } from "$lib/config/shortcuts";
import type {
	ShortcutAction,
	ShortcutCallback,
	ShortcutRegistration,
} from "$lib/types/input";

class InputManager {
	shiftPressed = $state(false);
	ctrlPressed = $state(false);
	altPressed = $state(false);
	metaPressed = $state(false);

	private shortcuts: ShortcutRegistration[] = [];

	/**
	 * Initializes the input manager and sets up global event listeners.
	 */
	constructor() {
		if (typeof window !== "undefined") {
			window.addEventListener("keydown", this.handleKeyDown.bind(this));
			window.addEventListener("keyup", this.handleKeyUp.bind(this));
			window.addEventListener("blur", this.resetModifiers.bind(this));
		}
	}

	/**
	 * Registers a global shortcut by action name defined in the centralized configuration.
	 * @param action The name of the action to register.
	 * @param callback The function to execute when the shortcut is triggered.
	 * @returns An unregister function to remove the shortcut.
	 */
	registerAction(action: ShortcutAction, callback: ShortcutCallback) {
		const config = defaultShortcuts[action];
		if (!config) {
			console.warn(`No shortcut configuration found for action: ${action}`);
			return () => {};
		}

		const registration: ShortcutRegistration = {
			...config,
			callback,
		};

		return this.register(registration);
	}

	/**
	 * Registers a manual global shortcut.
	 * @param registration The shortcut configuration and callback.
	 * @returns An unregister function to remove the shortcut.
	 */
	register(registration: ShortcutRegistration) {
		this.shortcuts.push(registration);
		return () => {
			this.shortcuts = this.shortcuts.filter((s) => s !== registration);
		};
	}

	/**
	 * Processes keydown events to match against registered shortcuts.
	 */
	private handleKeyDown(e: KeyboardEvent) {
		this.updateModifiers(e);

		const target = e.target as HTMLElement;
		const isInput =
			target?.tagName === "INPUT" ||
			target?.tagName === "TEXTAREA" ||
			target?.isContentEditable;

		for (const shortcut of this.shortcuts) {
			if (
				e.key.toLowerCase() === shortcut.key.toLowerCase() &&
				!!shortcut.ctrl === (e.ctrlKey || e.metaKey) &&
				!!shortcut.shift === e.shiftKey &&
				!!shortcut.alt === e.altKey
			) {
				// If typing in an input, ONLY allow shortcuts that use a modifier (Ctrl/Cmd/Alt)
				// or are specifically the Escape key. This prevents Shift+K from being
				// intercepted while typing a capital 'K'.
				if (isInput) {
					const hasModifier = e.ctrlKey || e.metaKey || e.altKey;
					if (!hasModifier && e.key !== "Escape") continue;
				}

				e.preventDefault();
				shortcut.callback(e);
				break;
			}
		}
	}

	/**
	 * Processes keyup events to update modifier states.
	 */
	private handleKeyUp(e: KeyboardEvent) {
		this.updateModifiers(e);
	}

	/**
	 * Synchronizes the internal modifier states with the current event state.
	 */
	private updateModifiers(e: KeyboardEvent | MouseEvent) {
		this.shiftPressed = e.shiftKey;
		this.ctrlPressed = e.ctrlKey || e.metaKey;
		this.altPressed = e.altKey;
		this.metaPressed = e.metaKey;
	}

	/**
	 * Resets all modifier states to false (typically on window blur).
	 */
	private resetModifiers() {
		this.shiftPressed = false;
		this.ctrlPressed = false;
		this.altPressed = false;
		this.metaPressed = false;
	}
}

export const inputManager = new InputManager();
