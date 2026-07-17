import type { MilkdownPlugin } from "@milkdown/ctx";
import { Plugin } from "@milkdown/prose/state";
import type { EditorView } from "@milkdown/prose/view";
import { $prose } from "@milkdown/utils";
import { openUrl } from "@tauri-apps/plugin-opener";

/**
 * Milkdown plugin that opens links in the default browser.
 */

export const linkOpenerPlugin: MilkdownPlugin = $prose(
	() =>
		new Plugin({
			props: {
				handleDOMEvents: {
					click(_view: EditorView, event: MouseEvent): boolean {
						const anchor = (
							event.target as HTMLElement | null
						)?.closest<HTMLAnchorElement>("a[href]");

						if (!anchor) {
							return false;
						}

						event.preventDefault();

						const rawHref = anchor.getAttribute("href");

						if (!rawHref) {
							return true;
						}

						const url = /^[a-z][a-z0-9+.-]*:\/\//i.test(rawHref)
							? rawHref
							: `https://${rawHref}`;

						void openUrl(url).catch((error) => {
							console.error("Failed to open URL:", error);
						});

						return true;
					},
				},
			},
		}),
);
