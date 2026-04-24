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
	altPressed = $state(false);
	metaPressed = $state(false);
	ctrlPressed = $state(false);
	shiftPressed = $state(false);

	/**
	 * Returns the 'Primary' modifier based on physical location (next to spacebar).
	 * Mac: Command (meta), Others: Ctrl (ctrlKey)
	 */
	get primaryPressed() {
		return sessionState.isMac ? this.metaPressed : this.ctrlPressed;
	}

	/**
	 * Returns the 'Secondary' modifier based on physical location (moving away from space).
	 * Mac: Option (alt), Others: Shift
	 */
	get secondaryPressed() {
		return sessionState.isMac ? this.altPressed : this.shiftPressed;
	}

	/**
	 * Returns the platform-specific label for the primary modifier.
	 */
	get primaryLabel() {
		return sessionState.isMac ? "⌘" : "Ctrl";
	}

	/**
	 * Returns the platform-specific label for the secondary modifier.
	 */
	get secondaryLabel() {
		return sessionState.isMac ? "⌥" : "Shift";
	}

	/** List of registered shortcut configurations. */
	private shortcuts: ShortcutRegistration[] = [];

	/**
	 * Initializes the input manager and sets up global event listeners.
	 */
	constructor() {
		if (typeof window !== "undefined") {
			window.addEventListener("keydown", this.handleKeyDown.bind(this), {
				capture: true,
			});
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
	 * Handles global keydown events, updates modifier states, and triggers matching shortcuts.
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

			const matchesPrimary =
				shortcut.primary === undefined ||
				!!shortcut.primary === this.primaryPressed;

			const matchesSecondary =
				shortcut.secondary === undefined ||
				!!shortcut.secondary === this.secondaryPressed;

			const matchesShift =
				shortcut.shift === undefined || !!shortcut.shift === this.shiftPressed;

			// Handle comma key (',') as a special case because it's the split delimiter
			const keys =
				shortcut.key === ","
					? [","]
					: shortcut.key.split(",").map((k) => k.trim());

			const matchesKey = keys.some((k) => {
				const lowerK = k.toLowerCase();
				return (
					e.key.toLowerCase() === lowerK ||
					e.code.toLowerCase() === lowerK ||
					e.code.toLowerCase() === `digit${lowerK}`
				);
			});

			if (matchesKey && matchesPrimary && matchesSecondary && matchesShift) {
				// If typing in an input, ONLY allow shortcuts that use a modifier
				// or are specifically the Escape key.
				if (isInput) {
					if (
						!this.primaryPressed &&
						!this.secondaryPressed &&
						!this.shiftPressed &&
						e.key !== "Escape"
					)
						continue;
				}

				const result = shortcut.callback(e);
				if (result !== false) {
					e.preventDefault();
					e.stopPropagation();
					e.stopImmediatePropagation();
					break;
				}
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
	updateModifiers(e: KeyboardEvent | MouseEvent) {
		this.altPressed = e.altKey;
		this.metaPressed = e.metaKey;
		this.ctrlPressed = e.ctrlKey;
		this.shiftPressed = e.shiftKey;
	}

	/**
	 * Resets all modifier states to false (typically on window blur).
	 */
	private resetModifiers() {
		this.altPressed = false;
		this.metaPressed = false;
		this.ctrlPressed = false;
		this.shiftPressed = false;
	}
}

export const inputManager = new InputManager();
