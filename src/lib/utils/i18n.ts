import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";

export const translations = writable<Record<string, string>>({});

export const locale = writable<string>("en");

export async function updateTranslations(newLocale: string) {
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
}

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
