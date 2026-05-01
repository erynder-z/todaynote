/**
 * Svelte action that synchronizes an HTML element's `lang` attribute
 * with the provided locale.
 */
export const lang = (node: HTMLElement, locale: string) => {
	node.lang = locale;

	return {
		/**
		 * Updates the attribute whenever the locale store changes.
		 */
		update(newLocale: string) {
			node.lang = newLocale;
		},
	};
};
