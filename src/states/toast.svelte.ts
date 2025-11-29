import type Toast from '$types/toast';
import { toast } from 'waku/components';

/**
 * Manages toast notifications in the application.
 */
class ToastState {
	/**
	 * Array of active toast notifications.
	 * @type {Toast[]}
	 */
	toasts: Toast[] = $state([]);

	/**
	 * Adds a new toast message to the list.
	 * @param {string} message - The text content of the toast.
	 * @param {'info' | 'warning' | 'error' | 'success'} [type='info'] - The type of the toast.
	 * @param {number | null} [timeout=null] - Optional time in milliseconds before removing the toast.
	 * @returns {void}
	 */
	addToast(
		message: string,
		type: 'info' | 'warning' | 'error' | 'success' = 'info',
		timeout: number | null = 3600
	): void {
		// This function is now a legacy function because waku-toast will be used
		// to display toasts in the future.
		// This function is now just a shim to keep the old interface working.
		toast({ title: message, type: type, timeout: timeout });
	}

	/**
	 * Removes a toast message by its ID.
	 * @param {number} id - The ID of the toast to remove.
	 * @returns {void}
	 */
	removeToast(id: number): void {
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}

	/**
	 * Clears all toast messages.
	 * @returns {void}
	 */
	clearToasts(): void {
		this.toasts = [];
	}
}

const toaster = new ToastState();
export default toaster;
export const addToast = toaster.addToast.bind(toaster);
