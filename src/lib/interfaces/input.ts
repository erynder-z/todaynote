import type { ShortcutAction, ShortcutCallback } from "../types/input";

export interface ShortcutConfig {
	key: string;
	primary?: boolean;
	secondary?: boolean;
	description?: string;
}

export interface ShortcutRegistration extends Partial<ShortcutConfig> {
	action?: ShortcutAction;
	callback: ShortcutCallback;
}
