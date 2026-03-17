/**
 * Centralized Input Manager for handling global keyboard shortcuts and tracking key states.
 */

import type { ShortcutRegistration } from "$lib/types/input";

class InputManager {
	shiftPressed = $state(false);
	ctrlPressed = $state(false);
	altPressed = $state(false);
	metaPressed = $state(false);

	private shortcuts: ShortcutRegistration[] = [];

	constructor() {
		if (typeof window !== "undefined") {
			window.addEventListener("keydown", this.handleKeyDown.bind(this));
			window.addEventListener("keyup", this.handleKeyUp.bind(this));
			window.addEventListener("blur", this.resetModifiers.bind(this));
		}
	}

	/**
	 * Registers a global shortcut.
	 */
	public register(registration: ShortcutRegistration) {
		this.shortcuts.push(registration);
		return () => {
			this.shortcuts = this.shortcuts.filter((s) => s !== registration);
		};
	}

	/**
	 * Handles the keydown event and triggers registered shortcuts if conditions are met.
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
	 * Handles the keyup event and updates modifier states.
	 */
	private handleKeyUp(e: KeyboardEvent) {
		this.updateModifiers(e);
	}

	/**
	 * Updates the state of modifier keys.
	 */
	private updateModifiers(e: KeyboardEvent | MouseEvent) {
		this.shiftPressed = e.shiftKey;
		this.ctrlPressed = e.ctrlKey || e.metaKey;
		this.altPressed = e.altKey;
		this.metaPressed = e.metaKey;
	}

	/**
	 * Resets all modifier key states.
	 */
	private resetModifiers() {
		this.shiftPressed = false;
		this.ctrlPressed = false;
		this.altPressed = false;
		this.metaPressed = false;
	}
}

export const inputManager = new InputManager();
