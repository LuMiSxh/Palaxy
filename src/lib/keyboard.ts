import type { SingleKey, KeyCombination } from '$types/keys';

type KeyHandler = {
	id: string;
	callback: (event: KeyboardEvent) => void | boolean;
};

// Convert a key combination string to lowercase for consistent matching
function normalizeKeyCombo(combo: KeyCombination | string): Lowercase<KeyCombination> {
	return combo.toLowerCase() as Lowercase<KeyCombination>;
}

class KeyboardManager {
	// Store multiple handlers per key combo in registration order
	private handlers: Map<KeyCombination, KeyHandler[]> = new Map();
	private excludeHandlers: Map<string, KeyHandler> = new Map();
	private enabled: boolean = true;

	constructor() {
		this.handleKeyDown = this.handleKeyDown.bind(this);
	}

	/**
	 * Register a key or key combination with a handler function
	 * @param keyCombo String like "ctrl+k" or "shift+alt+a"
	 * @param callback Function to execute when key combo is pressed
	 * @param id Optional ID to identify the handler. If not provided, a random ID is generated
	 * @returns The ID of the registered handler
	 */
	register(
		keyCombo: KeyCombination,
		callback: (event: KeyboardEvent) => void,
		id: string | undefined = undefined
	): string {
		// Normalize the key combo string
		keyCombo = normalizeKeyCombo(keyCombo);

		if (!this.handlers.has(keyCombo)) {
			this.handlers.set(keyCombo, []);
		}

		id = this.generateIdIfNotProvided(id);

		// Check if id is already in use
		if (this.handlers.get(keyCombo)!.find((h) => h.id === id)) {
			throw new Error(`Handler with ID "${id}" already exists`);
		}

		const handler: KeyHandler = { id, callback };

		const handlers = this.handlers.get(keyCombo)!;
		handlers.push(handler);

		return id;
	}

	/**
	 * Register a key handler that excludes certain keys and calls the callback for all others
	 * @param excludedKeys Array of keys to exclude
	 * @param callback Function to execute when a non-excluded key is pressed
	 * @param id Optional ID to identify the handler. If not provided, a random ID is generated
	 */
	registerExcept(
		excludedKeys: SingleKey[],
		callback: (event: KeyboardEvent) => void,
		id: string | undefined = undefined
	): string {
		id = this.generateIdIfNotProvided(id);

		// Create a wrapper function that checks if the key should be excluded
		const wrappedCallback = (event: KeyboardEvent) => {
			if (!this.enabled) return;

			if (excludedKeys.includes(event.code.toLowerCase() as SingleKey)) {
				return; // Skip excluded keys
			}

			event.stopPropagation();
			callback(event);
		};

		// Store the handler function so we can remove it later
		this.excludeHandlers.set(id, {
			id,
			callback: wrappedCallback,
		});

		// Add the event listener
		if (typeof window !== 'undefined') {
			window.addEventListener('keydown', wrappedCallback);
		}

		return id;
	}

	/**
	 * Unregister a key combo handler by id
	 */
	unregister(id: string): void {
		let deleted = false;
		// Try to remove a direct handler

		for (const [key, handlers] of this.handlers.entries()) {
			const index = handlers.findIndex((h) => h.id === id);
			if (index !== -1) {
				if (handlers.length === 1) {
					this.handlers.delete(key);
				} else {
					handlers.splice(index, 1);
				}
				deleted = true;
			}
		}

		// Try to remove an except handler if not already removed
		if (deleted) return;

		const exceptHandler = this.excludeHandlers.get(id);
		if (exceptHandler) {
			if (typeof window !== 'undefined') {
				window.removeEventListener('keydown', exceptHandler.callback);
			}
			this.excludeHandlers.delete(id);
		}
	}

	/**
	 * Register and unregister multiple handlers at once.
	 * For it to work, this function has to be called and its valued return in the mount of a component.
	 * @param handlers List of tuples with the parameters for the register function
	 * @param exceptHandlers List of tuples with the parameters for the registerExcept function
	 */
	smartRegister = (
		handlers: Parameters<typeof this.register>[],
		exceptHandlers: Parameters<typeof this.registerExcept>[] = []
	): (() => void) => {
		const ids: string[] = [];

		// Register all normal handlers
		for (const args of handlers) {
			const id = this.register(...args);
			ids.push(id);
		}

		// Register all except handlers
		for (const args of exceptHandlers) {
			const id = this.registerExcept(...args);
			ids.push(id);
		}

		// Return cleanup function
		return () => {
			ids.forEach((id) => this.unregister(id));
		};
	};

	/**
	 * Generate a random (unique) ID if not provided
	 */
	private generateIdIfNotProvided(id: string | undefined): string {
		if (!id) {
			let candidateId = '';
			let condition = true;

			// Keep generating IDs until we find a unique one
			do {
				candidateId = Math.random().toString(36).substring(2, 9);

				// Assume unique until proven otherwise
				let isUnique = true;

				// Check all registered key combos
				for (const handlers of this.handlers.values()) {
					if (handlers.some((h) => h.id === candidateId)) {
						isUnique = false;
						break;
					}
				}

				// If still unique, check excludes handlers
				if (isUnique && this.excludeHandlers.has(candidateId)) {
					isUnique = false;
				}

				// If unique after all checks, use this ID
				if (isUnique) {
					id = candidateId;
					condition = false;
				}
			} while (condition);
		}
		return id as string;
	}

	/**
	 * Handle keydown events
	 */
	private handleKeyDown(event: KeyboardEvent): void {
		if (!this.enabled) return;

		// Create the key combination string from the event
		const modifiers: string[] = [
			event.ctrlKey ? 'ctrl' : '',
			event.altKey ? 'alt' : '',
			event.shiftKey ? 'shift' : '',
			event.metaKey ? 'meta' : '',
		].filter(Boolean);

		const keyCombo = normalizeKeyCombo(
			modifiers.length > 0 ? `${modifiers.join('+')}+${event.code}` : event.code
		);

		// Check if we have handlers for this combination
		const handlers = this.handlers.get(keyCombo) || [];

		// Call all matching handlers in the order newest to oldest - Ignoring letter key combos when focused in input
		for (let i = handlers.length - 1; i >= 0; i--) {
			const isLetterKey = /^[KeyA-KeyZ]$/.test(event.code);
			const containsLetterKey = keyCombo.split('+').some((part) => /^[KeyA-KeyZ]$/.test(part));

			if (
				isLetterKey &&
				containsLetterKey &&
				(event.target instanceof HTMLInputElement ||
					event.target instanceof HTMLTextAreaElement ||
					(event.target instanceof HTMLElement && event.target.isContentEditable))
			) {
				continue; // Skip letter key combos when focused in input
			}

			if (handlers[i].callback(event) === true) {
				break;
			}
		}
	}

	/**
	 * Attach the keyboard handler to the window
	 * @returns A function to remove the handler
	 */
	mount(): () => void {
		if (typeof window !== 'undefined') {
			// Use the already bound handler from the constructor
			window.addEventListener('keydown', this.handleKeyDown);
		}

		// Return an unmount function that uses the same reference
		return () => {
			if (typeof window !== 'undefined') {
				window.removeEventListener('keydown', this.handleKeyDown);
			}
		};
	}
}

// Create and export a singleton instance
export const keyboard = new KeyboardManager();
