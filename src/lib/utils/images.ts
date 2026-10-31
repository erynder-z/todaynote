import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { sessionState } from "../stores/sessionState.svelte";

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
 * into a URL the webview can load.
 *
 * Notes keep relative paths on disk so they stay portable; only the
 * rendered DOM uses a loadable URL. Production builds use the asset
 * protocol on every platform.
 *
 * In dev, WebKitGTK 2.54+ refuses to load `asset://` subresources from
 * the http dev-server origin ("Unsafe attempt to load URL"), a
 * Linux-only problem, so local files are routed through Vite's `/@fs`
 * endpoint there instead (requires `server.fs.strict: false` in
 * vite.config). Dev builds on macOS and Windows keep using the asset
 * protocol.
 * https://github.com/tauri-apps/tauri/issues/16201
 */
export const resolveImageSrc = async (src: string): Promise<string> => {
	const dir = await getImagesDir();
	if (!dir || !src.startsWith("images/")) return src;

	const path = `${dir}/${src.slice("images/".length)}`;

	/* return convertFileSrc(`${dir}/${src.slice("images/".length)}`); */

	return import.meta.env.DEV && sessionState.isLinux
		? `/@fs${path}`
		: convertFileSrc(path);
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

/** Extensions accepted for stored image files. */
export const IMAGE_EXTENSIONS = [
	"png",
	"jpg",
	"jpeg",
	"gif",
	"webp",
	"bmp",
	"avif",
];

/**
 * Whether a file name carries an extension accepted for stored images.
 */
export const hasSupportedImageExtension = (name: string): boolean => {
	const ext = name.includes(".")
		? (name.split(".").pop() ?? "").toLowerCase()
		: "";
	return IMAGE_EXTENSIONS.includes(ext);
};

/**
 * Converts a `file://` URI into a local filesystem path. Returns null
 * for non-file URIs, remote hosts and malformed input.
 */
export const fileUriToPath = (uri: string): string | null => {
	if (!uri.startsWith("file://")) return null;

	try {
		const url = new URL(uri);
		if (url.host && url.host !== "localhost") return null;

		const path = decodeURIComponent(url.pathname);
		// Windows drive-letter paths arrive as "/C:/...".
		return /^\/[A-Za-z]:\//.test(path) ? path.slice(1) : path;
	} catch {
		return null;
	}
};

/**
 * Extracts local image file paths from clipboard text — a `text/uri-list`
 * payload and/or plain text — e.g. when a file was copied in the OS file
 * manager. Comments, remote URIs and non-image files are ignored.
 */
export const localImagePathsFromClipboardText = (
	uriList: string,
	plainText: string,
): string[] => {
	const candidates = uriList
		.split("\n")
		.map((line) => line.trim())
		.filter((line) => line.length > 0 && !line.startsWith("#"));

	const plain = plainText.trim();
	if (plain && !candidates.includes(plain)) candidates.push(plain);

	const paths: string[] = [];
	for (const candidate of candidates) {
		const path = candidate.startsWith("file://")
			? fileUriToPath(candidate)
			: candidate.startsWith("/")
				? candidate
				: null;

		if (path && hasSupportedImageExtension(path) && !paths.includes(path))
			paths.push(path);
	}

	return paths;
};
