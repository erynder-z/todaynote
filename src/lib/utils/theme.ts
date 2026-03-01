import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";
import type { ThemeInfo } from "../types/settings";

export const availableThemes = writable<ThemeInfo[]>([]);
export const currentTheme = writable<string>("light");

export async function updateTheme(themeId: string) {
	try {
		const colors: Record<string, string> = await invoke("get_theme_colors", {
			theme: themeId,
		});
		applyThemeColors(colors);
		currentTheme.set(themeId);
	} catch (error) {
		console.error("Failed to fetch theme colors:", error);
	}
}

export function applyThemeColors(colors: Record<string, string>) {
	const root = document.documentElement;
	Object.entries(colors).forEach(([property, value]) => {
		root.style.setProperty(property, value);
	});
}
