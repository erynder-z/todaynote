import { type Editor, editorViewCtx, parserCtx } from "@milkdown/kit/core";
import { NodeSelection, Selection } from "@milkdown/kit/prose/state";

/**
 * Service class for Milkdown editor operations.
 * Provides methods for navigating, updating, and focusing editor content.
 */
export class EditorService {
	constructor(private editor: Editor) {}

	private updateTimeout: ReturnType<typeof setTimeout> | null = null;

	/**
	 * Directs the editor to a specific thread by index.
	 * Finds the thread marker (!!!), moves the cursor to the end of that thread and scrolls to it.
	 */
	jumpToThreadByIndex(index: number) {
		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			const { doc } = view.state;

			const threadMarkers: { name: string; pos: number; endPos: number }[] = [];

			doc.descendants((node, pos) => {
				if (node.type.name === "thread_marker") {
					threadMarkers.push({
						name: node.textContent,
						pos,
						endPos: pos + node.nodeSize,
					});
				}
			});

			if (index < 0 || index >= threadMarkers.length) return;

			const thread = threadMarkers[index];
			const nextThread = threadMarkers[index + 1];

			// End of this thread's section:
			// either right before the next thread marker or end of document
			const cursorPos = nextThread ? nextThread.pos : doc.content.size;

			const resolvedPos = doc.resolve(cursorPos);
			const selection = Selection.near(resolvedPos, -1);

			// Move cursor
			view.dispatch(view.state.tr.setSelection(selection));

			// Scroll the thread marker itself
			requestAnimationFrame(() => {
				const dom = view.nodeDOM(thread.pos);

				if (dom instanceof HTMLElement) {
					dom.scrollIntoView({
						behavior: "smooth",
						block: "start",
					});
				}
			});

			view.focus();
		});
	}

	/**
	 * Updates the editor's content from a Markdown string and positions the cursor at the end.
	 * Debounced to prevent rapid successive updates from thrashing the editor.
	 */
	updateContent(markdown: string, onComplete?: () => void) {
		// Clear any pending update
		if (this.updateTimeout) clearTimeout(this.updateTimeout);

		this.updateTimeout = setTimeout(() => {
			this.editor.action((ctx) => {
				const view = ctx.get(editorViewCtx);
				const parser = ctx.get(parserCtx);
				const doc = parser(markdown);
				if (!doc) {
					if (onComplete) onComplete();
					return;
				}

				let tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc);

				// Ensure a trailing paragraph: milkdown needs a trailing empty
				// line for thread markers, and the caret needs a textblock
				// after a trailing image to rest in (otherwise the image
				// would end up node-selected).
				const lastChildType = doc.lastChild?.type.name;
				if (
					lastChildType === "thread_marker" ||
					lastChildType === "image-block"
				) {
					const paragraph = view.state.schema.nodes.paragraph.create();
					tr = tr.insert(tr.doc.content.size, paragraph);
				}

				// Position cursor at end and focus
				const selection = Selection.atEnd(tr.doc);
				view.dispatch(tr.setSelection(selection));
				view.focus();

				requestAnimationFrame(() => {
					view.dom.scrollIntoView({ block: "end", behavior: "smooth" });
				});

				// Call completion callback after DOM update
				requestAnimationFrame(() => {
					if (onComplete) onComplete();
				});
			});
		}, 50);
	}

	/**
	 * Inserts an image as a block node into the line after the one the
	 * caret is currently in, and places the caret after it. Returns false
	 * when the editor is not ready.
	 *
	 * Inserting through a transaction keeps undo history intact and flows
	 * through the regular editor change/auto-save path.
	 */
	insertImage(relativePath: string): boolean {
		let inserted = false;

		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			const { schema } = view.state;

			const type = schema.nodes["image-block"];
			if (!type) return;

			const { selection } = view.state;
			const $from = selection.$from;

			let insertPos: number;
			let replaceLine = false;

			if (selection instanceof NodeSelection && selection.node.isBlock) {
				// A block node is selected: insert right after it.
				insertPos = $from.pos + selection.node.nodeSize;
			} else if ($from.depth === 0) {
				// Gap cursor or another block-level position: insert at it.
				insertPos = $from.pos;
			} else if (
				$from.depth === 1 &&
				$from.parent.type.name === "paragraph" &&
				$from.parent.content.size === 0
			) {
				// The caret sits on a blank line: let the image take that
				// line instead of stacking below the blank one.
				replaceLine = true;
				insertPos = $from.before($from.depth);
			} else {
				// Insert after the top-level block containing the caret.
				insertPos = $from.after(1);
			}

			const image = type.create({
				src: relativePath,
				caption: "",
				size: "medium",
				align: null,
			});

			let tr = replaceLine
				? view.state.tr.replaceWith(
						insertPos,
						insertPos + $from.parent.nodeSize,
						image,
					)
				: view.state.tr.insert(insertPos, image);

			// Keep a paragraph after the image so the caret can rest there
			// and the next thread marker stays in its own block.
			const imageEnd = insertPos + image.nodeSize;
			const $after = tr.doc.resolve(imageEnd);
			if (!$after.nodeAfter || $after.parent.type.name === "thread_marker") {
				tr = tr.insert(imageEnd, schema.nodes.paragraph.create());
			}

			// Position the caret after the image; never inside the next
			// thread's marker.
			let caret = Selection.near(tr.doc.resolve(imageEnd), 1);
			if (caret.$from.parent.type.name === "thread_marker") {
				caret = Selection.near(tr.doc.resolve(imageEnd), -1);
			}

			view.dispatch(tr.setSelection(caret).scrollIntoView());
			inserted = true;
		});

		return inserted;
	}

	/**
	 * Focuses the editor instance.
	 */
	focus() {
		this.editor.action((ctx) => ctx.get(editorViewCtx).focus());
	}

	/**
	 * Removes focus from the editor instance.
	 */
	blur() {
		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			(view.dom as HTMLElement).blur();
		});
	}

	/**
	 * Focuses the editor and moves selection to the end.
	 */
	focusEnd() {
		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			view.focus();
			const tr = view.state.tr;
			const selection = Selection.atEnd(tr.doc);
			view.dispatch(tr.setSelection(selection));

			requestAnimationFrame(() => {
				view.dom.scrollIntoView({ block: "end", behavior: "smooth" });
			});
		});
	}
}
