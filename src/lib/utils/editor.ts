import { type Editor, editorViewCtx, parserCtx } from "@milkdown/core";
import { Selection } from "@milkdown/prose/state";

/**
 * Service class for Milkdown editor operations.
 * Provides methods for navigating, updating, and focusing editor content.
 */
export class EditorService {
	constructor(private editor: Editor) {}

	/**
	 * Directs the editor to a specific thread by index.
	 * Finds the heading and moves the cursor to the end of that thread.
	 */
	jumpToThreadByIndex(index: number) {
		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			const { doc } = view.state;

			// 1. Collect all top-level headings (level 1)
			const headings: { name: string; pos: number }[] = [];
			doc.descendants((node, pos) => {
				if (node.type.name === "heading" && node.attrs?.level === 1) {
					headings.push({ name: node.textContent.trim(), pos });
				}
			});

			// 2. Check if index is valid
			if (index < 0 || index >= headings.length) return;

			// 3. Target the boundary: either the next heading or end of doc
			const nextHeading = headings[index + 1];
			const jumpPos = nextHeading ? nextHeading.pos : doc.content.size;

			// 4. Update selection and scroll
			view.focus();
			const resolvedPos = view.state.doc.resolve(jumpPos);
			// Bias -1 ensures we land in the content BEFORE the next heading
			const selection = Selection.near(resolvedPos, -1);
			view.dispatch(view.state.tr.setSelection(selection).scrollIntoView());
		});
	}

	/**
	 * Updates the editor's content from a Markdown string and positions the cursor at the end.
	 */
	updateContent(markdown: string) {
		this.editor.action((ctx) => {
			const view = ctx.get(editorViewCtx);
			const parser = ctx.get(parserCtx);
			const doc = parser(markdown);
			if (!doc) return;

			let tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc);

			// Ensure trailing empty line for headings (Milkdown parser workaround)
			if (doc.lastChild?.type.name === "heading") {
				const paragraph = view.state.schema.nodes.paragraph.create();
				tr = tr.insert(tr.doc.content.size, paragraph);
			}

			// Position cursor at end and focus
			const selection = Selection.atEnd(tr.doc);
			view.dispatch(tr.setSelection(selection).scrollIntoView());
			view.focus();
		});
	}

	/**
	 * Focuses the editor instance.
	 */
	focus() {
		this.editor.action((ctx) => ctx.get(editorViewCtx).focus());
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
			view.dispatch(tr.setSelection(selection).scrollIntoView());
		});
	}
}
