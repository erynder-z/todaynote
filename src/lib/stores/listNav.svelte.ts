/**
 * Store for managing generic list navigation via keyboard.
 * Handles ArrowUp, ArrowDown, and Enter selection.
 */
export class ListNavigator {
	/** Current index of the highlighted item in the list. */
	index = $state(-1);

	/** Tracks whether the last index change was from keyboard or mouse navigation. */
	lastInputSource: "keyboard" | "mouse" | null = $state(null);

	/** Timestamp until which mouseenter events should be ignored (to prevent jump during keyboard scroll). */
	ignoreMouseEnterUntil = $state(0);

	/** Returns true if mouseenter events should currently be ignored (after keyboard navigation). */
	shouldIgnoreMouseEnter(): boolean {
		return Date.now() < this.ignoreMouseEnterUntil;
	}

	constructor(
		private getCount: () => number,
		private onSelect: (index: number) => void,
		private options: { wrap?: boolean } = { wrap: true },
	) {}

	/**
	 * Sets the index with a specified input source.
	 */
	setIndex(index: number, source: "keyboard" | "mouse"): void {
		this.index = index;
		this.lastInputSource = source;
	}

	/**
	 * Handles keyboard events for navigation and selection.
	 * Returns true if the key event was handled.
	 */
	handleKey(e: KeyboardEvent): boolean {
		const count = this.getCount();
		if (count === 0) return false;

		// Ignore mouseenter for 300ms after keyboard navigation to prevent jump
		this.ignoreMouseEnterUntil = Date.now() + 300;

		switch (e.key) {
			case "ArrowDown":
				e.preventDefault();
				if (this.options.wrap) {
					this.setIndex((this.index + 1) % count, "keyboard");
				} else {
					this.setIndex(Math.min(this.index + 1, count - 1), "keyboard");
				}
				return true;

			case "ArrowUp":
				e.preventDefault();
				if (this.options.wrap) {
					this.setIndex((this.index - 1 + count) % count, "keyboard");
				} else {
					this.setIndex(Math.max(this.index - 1, 0), "keyboard");
				}
				return true;

			case "Enter":
				if (this.index >= 0 && this.index < count) {
					e.preventDefault();
					this.onSelect(this.index);
					return true;
				}
				break;
		}

		return false;
	}

	/**
	 * Resets the highlighted index back to -1.
	 */
	reset() {
		this.index = -1;
		this.lastInputSource = null;
	}
}
