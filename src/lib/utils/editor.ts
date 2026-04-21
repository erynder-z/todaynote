import { type Editor, editorViewCtx } from "@milkdown/core";
import { Selection } from "@milkdown/prose/state";

/**
 * Directs the Milkdown editor to a specific named section.
 * Finds the heading and moves the cursor to the end of that section.
 */
export const jumpToSectionInEditor = (instance: Editor, name: string) => {
	instance.action((ctx) => {
		const view = ctx.get(editorViewCtx);
		const { doc } = view.state;

		// 1. Collect all top-level headings (level 1)
		const headings: { name: string; pos: number }[] = [];
		doc.descendants((node, pos) => {
			if (node.type.name === "heading" && node.attrs?.level === 1) {
				headings.push({ name: node.textContent.trim(), pos });
			}
		});

		// 2. Find the target heading index
		const targetIdx = headings.findIndex((h) => h.name === name);
		if (targetIdx === -1) return;

		// 3. Target the boundary: either the next heading or end of doc
		const nextHeading = headings[targetIdx + 1];
		const jumpPos = nextHeading ? nextHeading.pos : doc.content.size;

		// 4. Update selection and scroll
		view.focus();
		const resolvedPos = view.state.doc.resolve(jumpPos);
		// Bias -1 ensures we land in the content BEFORE the next heading
		const selection = Selection.near(resolvedPos, -1);
		view.dispatch(view.state.tr.setSelection(selection).scrollIntoView());
	});
};
