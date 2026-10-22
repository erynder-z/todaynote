import { editorViewOptionsCtx } from "@milkdown/kit/core";
import type { MilkdownPlugin } from "@milkdown/kit/ctx";
import type { Node as ProseMirrorNode } from "@milkdown/kit/prose/model";
import { Plugin } from "@milkdown/kit/prose/state";
import type { NodeView, NodeViewConstructor } from "@milkdown/kit/prose/view";
import type { MarkdownNode } from "@milkdown/kit/transformer";
import { $nodeSchema, $prose, $remark, $view } from "@milkdown/kit/utils";
import type { Node as MdastNode } from "unist";
import {
	buildImageFragment,
	IMAGE_ALIGNS,
	IMAGE_SIZES,
	parseImageFragment,
	resolveImageSrc,
	stripImageFragment,
} from "../utils/images.ts";

/**
 * First-class block-level image nodes for the editor.
 *
 * Notes store images as `![caption](images/<file>#<size>-<align>)` on their
 * own line. This plugin lifts any paragraph containing exactly one image
 * into a dedicated `image-block` node, so images behave like blocks:
 *
 * - clicking selects the whole image (no giant text caret inside a paragraph),
 * - arrow keys step over the image and backspace removes it as one unit,
 * - the image can be dragged to another position,
 * - size, alignment and caption are real node attributes instead of being
 *   re-parsed from the markdown src string on every interaction.
 *
 * The markdown format on disk is unchanged; existing notes keep working.
 */

/**
 * Returns whether an mdast node is a paragraph whose only child is an image.
 */
const isSingleImageParagraph = (node: MdastNode): boolean => {
	const children = (node as MdastNode & { children?: Array<{ type?: string }> })
		.children;

	return (
		node.type === "paragraph" &&
		Array.isArray(children) &&
		children.length === 1 &&
		children[0].type === "image"
	);
};

/**
 * Builds the `image-block` mdast node replacing a single-image paragraph.
 */
const toImageBlockNode = (paragraph: MdastNode): MdastNode => {
	const image = (paragraph as MdastNode & { children: MdastNode[] })
		.children[0] as MdastNode & { url?: string; alt?: string };

	return {
		type: "image-block",
		url: String(image.url ?? ""),
		alt: String(image.alt ?? ""),
		position: paragraph.position,
	} as MdastNode;
};

/**
 * Remark transform that replaces single-image paragraphs with
 * `image-block` nodes, which the schema below parses into block nodes.
 */
const remarkImageBlockPlugin = $remark("remarkImageBlock", () => () => {
	const walk = (node: MdastNode) => {
		const withChildren = node as MdastNode & { children?: MdastNode[] };
		if (!withChildren.children) return;

		for (const child of withChildren.children) walk(child);

		withChildren.children = withChildren.children.map((child) =>
			isSingleImageParagraph(child) ? toImageBlockNode(child) : child,
		);
	};

	return (tree: MdastNode) => {
		walk(tree);
	};
});

/**
 * Schema of the `image-block` node.
 *
 * `parseMarkdown` splits the markdown src into path + size/align attrs.
 * `toMarkdown` serializes back into the exact same markdown format.
 */
export const imageBlockSchema = $nodeSchema("image-block", () => ({
	inline: false,
	group: "block",
	selectable: true,
	draggable: true,
	atom: true,
	isolating: true,
	marks: "",
	priority: 100,
	attrs: {
		src: { default: "" },
		caption: { default: "" },
		size: { default: null },
		align: { default: null },
	},
	parseDOM: [
		{
			tag: 'img[data-type="image-block"]',
			getAttrs: (dom) => {
				if (!(dom instanceof HTMLElement)) return false;
				const src = dom.getAttribute("src") ?? "";
				const { size, align } = parseImageFragment(src);
				return {
					src: stripImageFragment(src),
					caption: dom.getAttribute("alt") ?? "",
					size,
					align,
				};
			},
		},
	],
	toDOM: (node: ProseMirrorNode) => [
		"img",
		{
			"data-type": "image-block",
			src: String(node.attrs.src ?? ""),
			alt: String(node.attrs.caption ?? ""),
		},
	],
	parseMarkdown: {
		match: (node) => node.type === "image-block",
		runner: (state, node, type) => {
			const url = String((node as MarkdownNode & { url?: string }).url ?? "");
			const alt = String(
				(node as MarkdownNode & { alt?: string }).alt ?? "",
			).trim();
			const { size, align } = parseImageFragment(url);

			state.addNode(type, {
				src: stripImageFragment(url),
				caption: alt,
				size,
				align,
			});
		},
	},
	toMarkdown: {
		match: (node) => node.type.name === "image-block",
		runner: (state, node) => {
			const src = String(node.attrs.src ?? "");
			const caption = String(node.attrs.caption ?? "");

			state.openNode("paragraph");
			state.addNode("image", undefined, undefined, {
				url: `${src}${buildImageFragment(node.attrs.size, node.attrs.align)}`,
				alt: caption,
			});
			state.closeNode();
		},
	},
}));

/**
 * Node view rendering an image block as a `<figure>` with an optional
 * `<figcaption>`. Relative `images/` srcs are resolved to asset-protocol
 * URLs here; the document itself is never modified.
 */
class ImageBlockView implements NodeView {
	readonly dom: HTMLElement;
	private readonly img: HTMLImageElement;
	private figcaption: HTMLElement | null = null;
	private resolvedSrc: string | null = null;

	constructor(private node: ProseMirrorNode) {
		this.dom = document.createElement("figure");
		this.dom.className = "img-figure";

		this.img = document.createElement("img");
		this.img.setAttribute("data-type", "image-block");
		this.img.addEventListener("load", () =>
			this.img.classList.remove("img-loading"),
		);
		this.img.addEventListener("error", () =>
			this.img.classList.remove("img-loading"),
		);
		this.dom.appendChild(this.img);

		this.applyAttrs();
	}

	update(node: ProseMirrorNode): boolean {
		if (node.type !== this.node.type) return false;
		this.node = node;
		this.applyAttrs();
		return true;
	}

	ignoreMutation(): boolean {
		return true;
	}

	private applyAttrs(): void {
		const caption = String(this.node.attrs.caption ?? "").trim();
		const size = this.node.attrs.size as string | null;
		const align = this.node.attrs.align as string | null;

		this.dom.classList.remove(...IMAGE_SIZES.map((s) => `img-size-${s}`));
		this.dom.classList.remove(...IMAGE_ALIGNS.map((a) => `img-align-${a}`));
		if (size) this.dom.classList.add(`img-size-${size}`);
		if (align) this.dom.classList.add(`img-align-${align}`);

		if (caption) {
			if (!this.figcaption) {
				this.figcaption = document.createElement("figcaption");
				this.dom.appendChild(this.figcaption);
			}
			this.figcaption.className = "img-caption";
			this.figcaption.textContent = caption;
		} else if (this.figcaption) {
			this.figcaption.remove();
			this.figcaption = null;
		}

		this.img.alt = caption;

		const src = String(this.node.attrs.src ?? "");
		if (src && src !== this.resolvedSrc) {
			this.resolvedSrc = src;
			this.img.classList.add("img-loading");
			if (src.startsWith("images/")) {
				void resolveImageSrc(src).then((resolved) => {
					if (String(this.node.attrs.src ?? "") === src)
						this.img.src = resolved;
				});
			} else {
				this.img.src = src;
			}
		}
	}
}

const imageBlockView = $view(imageBlockSchema.node, () => {
	const view: NodeViewConstructor = (node) => new ImageBlockView(node);
	return view;
});

/**
 * Keeps an empty paragraph below a trailing image so the caret can always
 * be positioned after it and typing continues there. Skipped in readonly
 * views, where the paragraph would only add dead space.
 */
const imageBlockTrailingParagraphPlugin = $prose((ctx) => {
	const options = ctx.get(editorViewOptionsCtx);

	return new Plugin({
		appendTransaction: (trs, _oldState, newState) => {
			if (options.editable && !options.editable(newState)) return null;
			if (!trs.some((tr) => tr.docChanged)) return null;
			if (newState.doc.lastChild?.type.name !== "image-block") return null;

			return newState.tr.insert(
				newState.doc.content.size,
				newState.schema.nodes.paragraph.create(),
			);
		},
	});
});

/**
 * All plugins required for block-level images. Spread into the editor's
 * plugin list.
 */
export const imageBlockPlugin: MilkdownPlugin[] = [
	remarkImageBlockPlugin,
	imageBlockSchema,
	imageBlockView,
	imageBlockTrailingParagraphPlugin,
].flat();
