import { type Writable, writable } from 'svelte/store';
import { type default as Toast } from '$types/toast';

export const toasts: Writable<Toast[]> = writable([]);

export function addToast(
	message: string,
	type: 'info' | 'warning' | 'error' | 'success' = 'info',
	timeout: number | null = null
): void {
	const id = Math.floor(Math.random() * 1000000);
	const toast: Toast = { id, message, type, timeout };

	toasts.update((toasts) => [...toasts, toast]);

	if (timeout) {
		setTimeout(() => {
			toasts.update((toasts) => toasts.filter((t) => t.id !== id));
		}, timeout);
	}
}
