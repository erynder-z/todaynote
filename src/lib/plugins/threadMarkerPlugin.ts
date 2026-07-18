import { textblockTypeInputRule } from "@milkdown/prose/inputrules";
import { $inputRule, $nodeSchema, $remark } from "@milkdown/utils";
import type { Node as MdastNode } from "unist";

/**
 * Remark plugin that transforms paragraphs starting with "!!! " into
 * dedicated `threadMarker` AST nodes. This runs during markdown parsing,
 * before ProseMirror ever sees the content.
 *
 * On serialization, the node is written back as `!!! <name>\n`.
 */
export const remarkThreadMarkerPlugin = $remark(
	"remarkThreadMarker",
	() => () => (tree: MdastNode) => {
		const root = tree as MdastNode & { children: MdastNode[] };
		if (!root.children) return;

		root.children = root.children.map(
			(node: MdastNode & { children?: Array<{ value?: string }> }) => {
				if (node.type !== "paragraph" || !node.children?.length) return node;

				// Check if the paragraph's first text child starts with "!!! "
				const firstChild = node.children[0];
				if (!firstChild?.value?.startsWith("!!! ")) return node;

				const name = firstChild.value.substring(4).trim();
				return {
					type: "threadMarker",
					data: { name },
					position: node.position,
				};
			},
		);
	},
);

/**
 * ProseMirror node schema for thread markers.
 *
 * - `parseMarkdown`: Matches the custom `threadMarker` AST node created by the remark plugin
 *   and creates a ProseMirror node whose text content IS the thread name (no "!!!").
 * - `toMarkdown`: Serializes back to `!!! <name>` in the markdown output.
 * - `toDOM`/`parseDOM`: Renders as a styled `<div>` with the thread name as editable text content.
 *
 * The "!!!" prefix is NEVER rendered — it only exists in the markdown source.
 * The node uses `content: 'text*'` so the thread name is directly editable
 * (like a heading), and `marks: ''` to prevent formatting inside thread markers.
 */
export const threadMarkerSchema = $nodeSchema("thread_marker", () => ({
	group: "block",
	content: "text*",
	marks: "",
	defining: true,
	parseDOM: [
		{
			tag: "div.thread-marker",
		},
	],
	toDOM: () => ["div", { class: "thread-marker" }, 0],
	parseMarkdown: {
		match: (node) => node.type === "threadMarker",
		runner: (state, node, type) => {
			const name = ((node as unknown as { data?: { name?: string } }).data
				?.name ?? "") as string;
			state.openNode(type);
			if (name) state.addText(name);
			state.closeNode();
		},
	},
	toMarkdown: {
		match: (node) => node.type.name === "thread_marker",
		runner: (state, node) => {
			// Write back as a paragraph containing "!!! <name>"
			state.openNode("paragraph");
			state.addNode("text", undefined, `!!! ${node.textContent}`);
			state.closeNode();
		},
	},
}));

/**
 * Input rule for live thread marker creation.
 * When the user types "!!! " (three bangs + space) at the start of a line,
 * the paragraph is converted into a thread_marker node. Any text typed
 * afterward becomes the thread name.
 */
export const threadMarkerInputRule = $inputRule((ctx) =>
	textblockTypeInputRule(/^!!!\s$/, threadMarkerSchema.type(ctx)),
);

/**
 * All thread marker plugins bundled for easy registration.
 * Register with: `editor.use(threadMarkerPlugin)`
 */
export const threadMarkerPlugin = [
	remarkThreadMarkerPlugin,
	threadMarkerSchema,
	threadMarkerInputRule,
].flat();
