import { sessionState } from "#lib/stores/sessionState.svelte.ts";

/**
 * Cheap visibility check that avoids per-node getComputedStyle() calls (each
 * of which can force a style recalc). Uses checkVisibility() where the
 * webview supports it, falling back to offsetParent/getClientRects(), which
 * are null/empty for display:none subtrees.
 */
const isElementVisible = (el: HTMLElement): boolean => {
	if (typeof el.checkVisibility === "function") {
		return el.checkVisibility({ checkVisibilityCSS: true });
	}

	return el.offsetParent !== null || el.getClientRects().length > 0;
};

/**
 * Class for finding text matches within the current view (find-in-page functionality)
 */
export class FindInElement {
	private targetElement: HTMLElement | null = null;
	private searchQuery: string = "";

	constructor(private getTargetElement: () => HTMLElement | null) {}

	/**
	 * Perform text search and return matching ranges
	 */
	search(query: string): Range[] {
		this.searchQuery = query;

		if (!query) return [];

		this.targetElement = this.getTargetElement();
		if (!this.targetElement) return [];

		return this.findTextMatches();
	}

	/**
	 * Find all text matches in the target element
	 */
	findTextMatches(): Range[] {
		if (!this.targetElement) return [];

		const query = this.searchQuery.toLowerCase();
		const ranges: Range[] = [];

		// Walk visible text nodes and find matches in a single pass
		const walker = document.createTreeWalker(
			this.targetElement,
			NodeFilter.SHOW_TEXT,
			{
				acceptNode(node) {
					const parent = node.parentElement;
					if (!parent) return NodeFilter.FILTER_REJECT;

					if (!isElementVisible(parent)) return NodeFilter.FILTER_REJECT;

					// Exclude search bar elements
					if (parent.closest(".search-in-view"))
						return NodeFilter.FILTER_REJECT;

					return NodeFilter.FILTER_ACCEPT;
				},
			},
		);

		let currentNode = walker.nextNode() as Text | null;
		while (currentNode) {
			const text = currentNode.textContent?.toLowerCase() || "";
			let startPos = text.indexOf(query);

			while (startPos !== -1) {
				const range = new Range();
				range.setStart(currentNode, startPos);
				range.setEnd(currentNode, startPos + query.length);
				ranges.push(range);
				startPos = text.indexOf(query, startPos + query.length);
			}

			currentNode = walker.nextNode() as Text | null;
		}

		return ranges;
	}
}

/**
 * Determine whether find-in-note can be used in the current context.
 */
const canShowFindInNote = () => {
	const isNoteShown =
		sessionState.todayNoteContent !== null && sessionState.activePopup === null;

	return isNoteShown || sessionState.activePopup === "threadAggregation";
};

/**
 * Focus and select the find-in-note search input.
 */
const focusInput = () => {
	const input = document.querySelector<HTMLInputElement>(
		".search-in-view .search-input",
	);

	input?.focus();
	input?.select();
};

/**
 * Open the find-in-note panel or focus its search input if already open.
 */
export const toggleFindInNote = () => {
	if (!canShowFindInNote()) return;

	if (sessionState.showFindInNote) {
		focusInput();
		return;
	}

	sessionState.showFindInNote = true;
};
