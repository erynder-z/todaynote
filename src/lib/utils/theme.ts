import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";
import type { ThemeInfo } from "../types/settings";

/**
 * Holds the list of themes currently available in the application.
 */
export const availableThemes = writable<ThemeInfo[]>([]);

/**
 * The currently active theme ID (e.g., 'blind-spot', 'catppuccin-mocha').
 */
export const currentTheme = writable<string>("blind-spot");

/**
 * Fetches the specific color configuration for a theme and applies it to the UI.
 */
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

/**
 * Dynamically applies theme colors by setting CSS variables on the document root.
 */
export function applyThemeColors(colors: Record<string, string>) {
	const root = document.documentElement;
	Object.entries(colors).forEach(([property, value]) => {
		root.style.setProperty(property, value);
	});
}
