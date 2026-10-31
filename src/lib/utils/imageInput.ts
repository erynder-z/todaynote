import { open } from "@tauri-apps/plugin-dialog";
import { sessionState } from "../stores/sessionState.svelte";
import { toast } from "../stores/toast.svelte";
import {
	hasSupportedImageExtension,
	IMAGE_EXTENSIONS,
	localImagePathsFromClipboardText,
} from "./images.ts";
import { notesService } from "./notes";

/**
 * Whether a file is an image type the backend accepts. The MIME type
 * wins; the file name's extension is the fallback.
 */
export const isSupportedImageFile = (file: File): boolean => {
	const mimeExt = file.type.startsWith("image/")
		? file.type.slice("image/".length).split(";")[0].trim().toLowerCase()
		: "";
	const ext = mimeExt === "jpeg" ? "jpg" : mimeExt || "";
	if (IMAGE_EXTENSIONS.includes(ext)) return true;
	return hasSupportedImageExtension(file.name);
};

/**
 * Inserts an image into the note at the caret through the live editor
 * document, so the change flows through the regular editor update/auto-save path.
 */
export const insertImage = (relativePath: string): boolean => {
	const service = sessionState.editorService;

	if (!service?.insertImage(relativePath)) {
		toast.error("image.insert_error");
		return false;
	}

	toast.success("image.success");
	return true;
};

/**
 * Opens the OS file picker and inserts the selected image at the cursor.
 */
export const pickLocalImage = async (): Promise<void> => {
	const selection = await open({
		multiple: false,
		directory: false,
		filters: [{ name: "Images", extensions: IMAGE_EXTENSIONS }],
	});

	if (typeof selection !== "string") return;

	const relative = await notesService.addImageFromPath(selection);

	if (!relative) {
		toast.error("image.error");
		return;
	}

	insertImage(relative);
};

/**
 * Reads a file as a base64 string (without the data-URL prefix).
 */
const fileToBase64 = (file: File): Promise<string> =>
	new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => {
			const result = String(reader.result ?? "");
			resolve(result.slice(result.indexOf(",") + 1));
		};
		reader.onerror = () => reject(reader.error);
		reader.readAsDataURL(file);
	});

/**
 * Stores a pasted image file in the notes images directory and
 * returns the stored path relative to the notes folder.
 */
const saveImageFile = async (file: File): Promise<string | null> =>
	notesService.addImageFromBytes(
		file.name || "image",
		file.type,
		await fileToBase64(file),
	);

/**
 * Collects the image files of a clipboard event, from `files` or, when
 * that is empty, from the item list.
 */
const clipboardImageFiles = (e: ClipboardEvent): File[] => {
	const data = e.clipboardData;
	if (!data) return [];

	const files = Array.from(data.files ?? []);
	if (files.length > 0) return files.filter(isSupportedImageFile);

	return Array.from(data.items ?? [])
		.filter((item) => item.kind === "file")
		.map((item) => item.getAsFile())
		.filter(
			(file): file is File => file !== null && isSupportedImageFile(file),
		);
};

/**
 * Paste handler for the editor: inserts clipboard images at the caret
 * instead of letting ProseMirror turn them into text or links.
 *
 * Two clipboard shapes are recognized: actual image payloads (files),
 * and `file://` URIs of local image files, which is what OS file managers
 * put on the clipboard when a file is copied — those are copied into the
 * images directory and embedded instead of pasted as a link.
 *
 * Attach with capture so ProseMirror's own paste handler never sees them.
 */
export const handleEditorPaste = (e: ClipboardEvent): void => {
	const intercept = () => {
		e.preventDefault();
		e.stopPropagation();
	};

	const images = clipboardImageFiles(e);
	if (images.length > 0) {
		intercept();

		void (async () => {
			for (const file of images) {
				const relative = await saveImageFile(file);
				if (relative) insertImage(relative);
				else toast.error("image.error");
			}
		})();
		return;
	}

	const data = e.clipboardData;
	const paths = data
		? localImagePathsFromClipboardText(
				data.getData("text/uri-list"),
				data.getData("text/plain"),
			)
		: [];
	if (paths.length === 0) return;

	intercept();

	void (async () => {
		for (const path of paths) {
			const relative = await notesService.addImageFromPath(path);
			if (relative) insertImage(relative);
			else toast.error("image.error");
		}
	})();
};
