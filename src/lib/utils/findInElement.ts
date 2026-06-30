import { sessionState } from "$lib/stores/sessionState.svelte";

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

		const textNodes: Text[] = [];

		// Collect all visible text nodes
		const walker = document.createTreeWalker(
			this.targetElement,
			NodeFilter.SHOW_TEXT,
			{
				acceptNode(node) {
					const parent = node.parentElement;
					if (!parent) return NodeFilter.FILTER_REJECT;

					const style = window.getComputedStyle(parent);
					if (style.display === "none" || style.visibility === "hidden")
						return NodeFilter.FILTER_REJECT;

					// Exclude search bar elements
					if (parent.closest(".search-in-view"))
						return NodeFilter.FILTER_REJECT;

					return NodeFilter.FILTER_ACCEPT;
				},
			},
		);

		let currentNode = walker.nextNode();
		while (currentNode) {
			textNodes.push(currentNode as Text);
			currentNode = walker.nextNode();
		}

		// Find matches
		const query = this.searchQuery.toLowerCase();
		const ranges: Range[] = [];

		for (const node of textNodes) {
			const text = node.textContent?.toLowerCase() || "";
			let startPos = text.indexOf(query);

			while (startPos !== -1) {
				const range = new Range();
				range.setStart(node, startPos);
				range.setEnd(node, startPos + query.length);
				ranges.push(range);
				startPos = text.indexOf(query, startPos + query.length);
			}
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
