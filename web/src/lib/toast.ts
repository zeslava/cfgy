// Store тостов и функции показа
import { writable } from 'svelte/store';

export type ToastVariant = 'success' | 'error' | 'info' | 'warning';

export interface ToastItem {
	id: string;
	message: string;
	variant?: ToastVariant;
	duration?: number;
}

function createToastStore() {
	const { subscribe, update } = writable<ToastItem[]>([]);

	function add(item: ToastItem) {
		update((list) => [...list, item]);
	}

	function remove(id: string) {
		update((list) => list.filter((t) => t.id !== id));
	}

	function showToast(
		message: string,
		variant: ToastVariant = 'info',
		duration = 5000
	) {
		const id = `toast-${Date.now()}-${Math.random()}`;
		add({ id, message, variant, duration });
		if (duration > 0) {
			setTimeout(() => remove(id), duration);
		}
	}

	return {
		subscribe,
		remove,
		showToast,
		showSuccess: (message: string) => showToast(message, 'success'),
		showError: (message: string) => showToast(message, 'error'),
		showInfo: (message: string) => showToast(message, 'info'),
		showWarning: (message: string) => showToast(message, 'warning')
	};
}

export const toastStore = createToastStore();
