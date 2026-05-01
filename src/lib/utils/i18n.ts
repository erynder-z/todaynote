import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";
import type { LocaleInfo } from "$lib/types/locale";

/**
 * Holds the current key-value pairs for translations.
 */
export const translations = writable<Record<string, string>>({});

/**
 * The currently active locale ID (e.g., 'en', 'de').
 */
export const locale = writable<string>("en");

/**
 * List of locales supported by the application.
 */
export const availableLocales = writable<LocaleInfo[]>([]);

/**
 * Fetches new translation data from the backend and updates the stores.
 */
export const updateTranslations = async (newLocale: string) => {
	try {
		const newTranslations: Record<string, string> = await invoke(
			"get_translations",
			{
				locale: newLocale,
			},
		);
		translations.set(newTranslations);
		locale.set(newLocale);
	} catch (error) {
		console.error("Failed to fetch translations:", error);
	}
};

/**
 * A derived store providing a translation function 't' that supports variable interpolation.
 * Usage: $t('key', { variable: 'value' })
 */
export const t = derived(translations, ($translations) => {
	return (key: string, vars: Record<string, string | number> = {}) => {
		let text = $translations[key] || key;

		Object.keys(vars).forEach((v) => {
			const regex = new RegExp(`{{${v}}}`, "g");
			text = text.replace(regex, String(vars[v]));
		});

		return text;
	};
});
