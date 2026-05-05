import type { Toast, ToastType } from "$lib/types/ui";

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

	remove(id: string) {
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}

	success(message: string, duration?: number) {
		this.show(message, "success", duration);
	}

	error(message: string, duration?: number) {
		this.show(message, "error", duration);
	}

	warning(message: string, duration?: number) {
		this.show(message, "warning", duration);
	}

	info(message: string, duration?: number) {
		this.show(message, "info", duration);
	}
}

export const toast = new ToastStore();
