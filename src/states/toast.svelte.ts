import type Toast from '$types/toast';

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
		const id = Math.floor(Math.random() * 1000000);
		const toast: Toast = { id, message, type, timeout };

		this.toasts = [...this.toasts, toast];

		if (timeout) {
			setTimeout(() => {
				this.removeToast(id);
			}, timeout);
		}
	}

	/**
	 * Removes a toast message by its ID.
	 * @param {number} id - The ID of the toast to remove.
	 * @returns {void}
	 */
	removeToast(id: number): void {
		this.toasts = this.toasts.filter(t => t.id !== id);
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
