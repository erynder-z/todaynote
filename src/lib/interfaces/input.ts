import type { ShortcutCallback } from "../types/input";

export interface ShortcutConfig {
	key: string;
	primary?: boolean;
	secondary?: boolean;
	shift?: boolean;
	description?: string;
}

export interface ShortcutRegistration extends ShortcutConfig {
	callback: ShortcutCallback;
}
