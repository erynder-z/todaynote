import { invoke } from "@tauri-apps/api/core";
import { MarkdownRenderCache } from "./renderCache";

const renderCacheSize = 500;
const renderCache = new MarkdownRenderCache<string, string>(renderCacheSize);

export const readNoteContent = async (path: string) => {
	try {
		const content = (await invoke("read_note_content", { path })) as string;
		return content;
	} catch (error) {
		console.error(`Error reading note content from ${path}:`, error);
		return null;
	}
};

export const renderMarkdown = async (markdown: string) => {
	if (!markdown || !markdown.trim()) return "&nbsp;";

	// Return from cache if string is cached
	const cached = renderCache.get(markdown);
	if (cached !== undefined) return cached;

	try {
		const html = (await invoke("render_markdown", { markdown })) as string;

		// Store the new result in cache
		renderCache.set(markdown, html);
		return html;
	} catch (error) {
		console.error("Error rendering markdown:", error);
		return markdown;
	}
};

export const saveNoteContent = async (path: string, content: string) => {
	try {
		await invoke("save_note_content", { path, content });
		return true;
	} catch (error) {
		console.error(`Error saving note content to ${path}:`, error);
		return false;
	}
};
