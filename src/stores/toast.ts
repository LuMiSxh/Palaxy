/**
 * A writable store holding an array of Toast objects.
 */
import { type Writable, writable } from 'svelte/store';
import { type default as Toast } from '$types/toast';

/**
 * Holds the current list of toasts.
 */
export const toasts: Writable<Toast[]> = writable([]);

/**
 * Adds a new toast message to the list.
 * @param {string} message - The text content of the toast.
 * @param {'info' | 'warning' | 'error' | 'success'} [type='info'] - The type of the toast.
 * @param {number | null} [timeout=null] - Optional time in milliseconds before removing the toast.
 * @returns {void}
 */
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
