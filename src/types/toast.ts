/**
 * Represents a toast message displayed in the application.
 */
export default interface Toast {
	/**
	 * A unique numeric identifier for the toast.
	 */
	id: number;

	/**
	 * The message text to be displayed.
	 */
	message: string;

	/**
	 * The type of the toast, indicating its status or severity.
	 */
	type: 'success' | 'error' | 'warning' | 'info';

	/**
	 * The duration in milliseconds before the toast is dismissed, or null for no timeout.
	 */
	timeout: number | null;
}
