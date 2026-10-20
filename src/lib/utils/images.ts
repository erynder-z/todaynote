import { convertFileSrc, invoke } from "@tauri-apps/api/core";

/**
 * Cached absolute path of the notes images directory.
 */
let imagesDirPromise: Promise<string> | null = null;

/**
 * Resolves the absolute images directory from the backend (cached).
 */
const getImagesDir = () => {
	if (!imagesDirPromise) {
		imagesDirPromise = invoke<string>("get_images_dir").catch((error) => {
			console.error("Failed to resolve images directory:", error);
			return "";
		});
	}
	return imagesDirPromise;
};

/**
 * Resolves a relative image path used in note markdown (`images/<file>`)
 * into a URL the webview can load through Tauri's asset protocol.
 *
 * Notes keep relative paths on disk so they stay portable; only the
 * rendered DOM uses the asset URL.
 */
export const resolveImageSrc = async (src: string): Promise<string> => {
	const dir = await getImagesDir();
	if (!dir || !src.startsWith("images/")) return src;
	return convertFileSrc(`${dir}/${src.slice("images/".length)}`);
};

/** Display sizes an embedded image can be set to (Gmail-style). */
export type ImageSize = "small" | "medium" | "large";

/** Horizontal alignment an embedded image can be set to. */
export type ImageAlign = "left" | "center" | "right";

export const IMAGE_SIZES: ImageSize[] = ["small", "medium", "large"];
export const IMAGE_ALIGNS: ImageAlign[] = ["left", "center", "right"];

/**
 * Parsed image src fragment, e.g. `#medium-center`.
 * Either part may be absent; `left` is the implicit default alignment.
 */
export type ImageFragment = {
	size: ImageSize | null;
	align: ImageAlign | null;
};

/**
 * Removes the trailing size/alignment fragment from a markdown image src.
 */
export const stripImageFragment = (src: string): string =>
	src.replace(/#[a-z-]+$/, "");

/**
 * Extracts the trailing size/alignment fragment from a markdown image src.
 * Accepts `#<size>`, `#<align>` and `#<size>-<align>`.
 */
export const parseImageFragment = (src: string): ImageFragment => {
	const match = /#([a-z-]+)$/.exec(src);
	if (!match) return { size: null, align: null };

	const parts = match[1].split("-");
	const size = IMAGE_SIZES.includes(parts[0] as ImageSize)
		? (parts[0] as ImageSize)
		: null;
	const alignRaw = parts.length > 1 ? parts[1] : size ? null : parts[0];
	const align = IMAGE_ALIGNS.includes(alignRaw as ImageAlign)
		? (alignRaw as ImageAlign)
		: null;

	return { size, align };
};

/**
 * Builds a fragment string from size and alignment, omitting defaults.
 */
export const buildImageFragment = (
	size: ImageSize | null,
	align: ImageAlign | null,
): string => {
	const parts: string[] = [];
	if (size) parts.push(size);
	if (align && align !== "left") parts.push(align);
	return parts.length ? `#${parts.join("-")}` : "";
};
