export function lang(node: HTMLElement, locale: string) {
	node.lang = locale;

	return {
		update(newLocale: string) {
			node.lang = newLocale;
		},
	};
}
