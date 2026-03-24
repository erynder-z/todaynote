/**
 * Creates a focus trap for the given node.
 * Restricts keyboard focus to a specific area of the page.
 * Re-calculates focusable elements on each Tab press to support dynamic content.
 */
export const focusTrap = (node: HTMLElement) => {
	const selector =
		'a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), details, [tabindex]:not([tabindex="-1"])';

	/**
	 * Handles the keydown event for the focus trap.
	 */
	function handleKeydown(event: KeyboardEvent) {
		if (event.key !== "Tab") return;

		const focusableElements = node.querySelectorAll<HTMLElement>(selector);

		if (focusableElements.length === 0) {
			event.preventDefault();
			return;
		}

		const firstElement = focusableElements[0];
		const lastElement = focusableElements[focusableElements.length - 1];

		if (event.shiftKey) {
			if (document.activeElement === firstElement) {
				lastElement.focus();
				event.preventDefault();
			}
		} else {
			if (document.activeElement === lastElement) {
				firstElement.focus();
				event.preventDefault();
			}
		}
	}

	document.addEventListener("keydown", handleKeydown);

	return {
		destroy() {
			document.removeEventListener("keydown", handleKeydown);
		},
	};
};
