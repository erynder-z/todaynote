import { defaultShortcuts } from "$lib/config/shortcuts";
import type {
	ShortcutAction,
	ShortcutCallback,
	ShortcutRegistration,
} from "$lib/types/input";
import { sessionState } from "./sessionState.svelte";

/**
 * Centralized Input Manager for handling global keyboard shortcuts and tracking key states.
 */
class InputManager {
	shiftPressed = $state(false);
	ctrlPressed = $state(false);
	altPressed = $state(false);
	metaPressed = $state(false);

	/**
	 * Returns the platform-specific label for the Alt/Option key.
	 */
	get superLabel() {
		return sessionState.isMac ? "⌥" : "Super";
	}

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
	 * Registers a shortcut action based on the central configuration.
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
	 * Registers multiple shortcut actions at once.
	 */
	registerActions(actions: Partial<Record<ShortcutAction, ShortcutCallback>>) {
		const unregisters = Object.entries(actions).map(([action, callback]) =>
			this.registerAction(action as ShortcutAction, callback),
		);
		return () => {
			for (const unregister of unregisters) unregister();
		};
	}

	/**
	 * Low-level method to register a manual shortcut.
	 */
	register(registration: ShortcutRegistration) {
		this.shortcuts.push(registration);
		return () => {
			this.shortcuts = this.shortcuts.filter((s) => s !== registration);
		};
	}

	/**
	 * Processes keydown events to match against registered shortcuts.
	 * Favoring the most recently registered shortcuts by iterating backwards.
	 */
	private handleKeyDown(e: KeyboardEvent) {
		this.updateModifiers(e);

		const target = e.target as HTMLElement;
		const isInput =
			target?.tagName === "INPUT" ||
			target?.tagName === "TEXTAREA" ||
			target?.isContentEditable;

		for (let i = this.shortcuts.length - 1; i >= 0; i--) {
			const shortcut = this.shortcuts[i];
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
