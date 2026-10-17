import type { Toast, ToastAction } from "#lib/interfaces/ui.ts";
import type { ToastType } from "#lib/types/ui.ts";

class ToastStore {
	toasts = $state<Toast[]>([]);

	show(message: string, type: ToastType = "info", duration = 3000) {
		const id = crypto.randomUUID();
		const toast: Toast = { id, message, type, duration };
		this.toasts.push(toast);

		if (duration > 0) {
			setTimeout(() => {
				this.remove(id);
			}, duration);
		}
	}

	/**
	 * Shows a toast with an interactive action button.
	 */
	showWithAction(
		message: string,
		action: ToastAction,
		type: ToastType = "info",
		duration = 0,
	) {
		const id = crypto.randomUUID();
		const toast: Toast = { id, message, type, duration, action };
		this.toasts.push(toast);

		if (duration > 0) {
			setTimeout(() => {
				this.remove(id);
			}, duration);
		}
	}

	/**
	 * Removes the current toast from the queue
	 */
	remove(id: string) {
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}

	/**
	 * Shows a toast with a success message
	 */
	success(message: string, duration?: number) {
		this.show(message, "success", duration);
	}

	/**
	 * Shows a toast with an error message
	 */
	error(message: string, duration?: number) {
		this.show(message, "error", duration);
	}

	/**
	 * Shows a toast with a warning message
	 */
	warning(message: string, duration?: number) {
		this.show(message, "warning", duration);
	}

	/**
	 * Shows a toast with an info message
	 */
	info(message: string, duration?: number) {
		this.show(message, "info", duration);
	}
}

export const toast = new ToastStore();
