/**
 * Service for observing DOM mutations
 */
export class DOMObserver {
	private observer: MutationObserver | null = null;
	private callback: (() => void) | null = null;

	/**
	 * Start observing DOM mutations
	 */
	observe(target: HTMLElement, callback: () => void) {
		this.disconnect();

		this.callback = callback;
		this.observer = new MutationObserver(() => {
			if (this.callback) this.callback();
		});

		this.observer.observe(target, {
			childList: true,
			subtree: true,
			characterData: true,
		});
	}

	/**
	 * Disconnect the observer
	 */
	disconnect() {
		if (this.observer) {
			this.observer.disconnect();
			this.observer = null;
		}
		this.callback = null;
	}
}
