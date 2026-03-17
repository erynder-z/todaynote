export type ShortcutCallback = (e: KeyboardEvent) => void | Promise<void>;

export type ShortcutRegistration = {
	key: string;
	ctrl?: boolean;
	shift?: boolean;
	alt?: boolean;
	meta?: boolean;
	callback: ShortcutCallback;
	description?: string;
};
