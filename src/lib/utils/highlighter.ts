/**
 * Class for managing CSS Custom Highlights API
 */
export class Highlighter {
	private resultsHighlight: Highlight | null = null;
	private activeHighlight: Highlight | null = null;

	/**
	 * Get or create highlight instances
	 */
	getHighlights(): {
		resultsHighlight: Highlight;
		activeHighlight: Highlight;
	} | null {
		if (
			typeof CSS === "undefined" ||
			!CSS.highlights ||
			typeof window.Highlight === "undefined"
		) {
			return null;
		}

		if (!this.resultsHighlight) {
			this.resultsHighlight = new window.Highlight();
			this.activeHighlight = new window.Highlight();
			CSS.highlights.set("search-results", this.resultsHighlight);
			CSS.highlights.set("current-search-result", this.activeHighlight);
		}

		if (!this.resultsHighlight || !this.activeHighlight) return null;

		return {
			resultsHighlight: this.resultsHighlight,
			activeHighlight: this.activeHighlight,
		};
	}

	/**
	 * Clear all highlights
	 */
	clearHighlights() {
		const root = document.documentElement;
		root.style.setProperty("--search-match-bg", "transparent");
		root.style.setProperty("--search-current-bg", "transparent");
		root.style.setProperty("--search-current-fg", "inherit");

		const h = this.getHighlights();
		if (h) {
			try {
				h.resultsHighlight.clear();
				h.activeHighlight.clear();
			} catch (e) {
				console.error("Failed to clear highlights:", e);
			}
		}
	}

	/**
	 * Apply highlights to ranges
	 */
	applyHighlights(ranges: Range[], currentIndex: number) {
		const h = this.getHighlights();
		if (!h) return;

		try {
			const root = document.documentElement;
			root.style.setProperty(
				"--search-match-bg",
				"color-mix(in srgb, var(--accent) 35%, transparent)",
			);
			root.style.setProperty("--search-current-bg", "var(--accent)");
			root.style.setProperty("--search-current-fg", "var(--accent-text)");

			h.resultsHighlight.clear();
			h.activeHighlight.clear();

			if (ranges.length === 0) return;

			for (const range of ranges) {
				h.resultsHighlight.add(range);
			}

			const activeRange = ranges[currentIndex];
			if (activeRange) {
				h.activeHighlight.add(activeRange);
			}
		} catch (e) {
			console.error("Failed to apply CSS highlights:", e);
		}
	}

	/**
	 * Clean up highlight resources
	 */
	cleanup() {
		if (typeof CSS !== "undefined" && CSS.highlights) {
			try {
				CSS.highlights.delete("search-results");
				CSS.highlights.delete("current-search-result");
			} catch (e) {
				console.error("Failed to cleanup highlights:", e);
			}
		}
	}
}
