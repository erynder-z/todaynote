import { writable } from "svelte/store";
import type { ThemeInfo } from "$lib/interfaces/settings";

/**
 * Holds the list of themes currently available in the application.
 */
export const availableThemes = writable<ThemeInfo[]>([]);

/**
 * The currently active theme ID (e.g., 'blind-spot', 'catppuccin-mocha').
 */
export const currentTheme = writable<string>("blind-spot");

/**
 * Dynamically applies theme colors by setting CSS variables on the document root.
 */
export const applyThemeColors = (colors: Record<string, string>) => {
	const root = document.documentElement;
	Object.entries(colors).forEach(([property, value]) => {
		root.style.setProperty(property, value);
	});
};
