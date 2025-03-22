import type { KeyCombination } from '$lib/keyboard';

class KeyHint {
	private keys: [KeyCombination, string][] = $state([]);

	constructor() {
	}

	/**
	 * Adds a key combination with its hint. If the key combination already exists, overrides its hint.
	 * @param key The key combination to add
	 * @param hint The hint to display for this key combination
	 */
	addKey(key: KeyCombination, hint: string): this {
		// Check if the key combination already exists
		const existingIndex = this.keys.findIndex(k => k[0] === key);

		if (existingIndex >= 0) {
			// If exists, override the hint
			this.keys = this.keys.map(k => k[0] === key ? [key, hint] : k);

		} else {
			// If not exists, add a new key combination
			this.keys = [...this.keys, [key, hint]];
		}

		return this;
	}

	/**
	 * Removes a key combination and its hint.
	 * @param key The key combination
	 * @returns This instance
	 */
	removeKey(key: KeyCombination): this {
		this.keys = this.keys.filter(k => k[0] !== key);
		return this;
	}

	/**
	 * Adds multiple key combinations with their hints. Returns a function to remove the added key combinations.
	 @param keyhints An array of key combinations and their hints
	 * @returns A function to remove the added key combinations
	 */
	smartAdd(keyhints: Parameters<typeof this.addKey>[]): () => void {
		// Get all current key hints
		const currentKeys = this.keys;

		// Add the new key hints
		keyhints.forEach(([key, hint]) => this.addKey(key, hint));

		// Return a function to remove the added key hints and restore the previous ones
		return () => {
			keyhints.forEach(([key]) => this.removeKey(key));

			this.keys = currentKeys;
		};
	}

	clear(): void {
		this.keys = [];
	}

	set(keys: [KeyCombination, string][]): void {
		this.keys = keys;
	}

	get(): [KeyCombination, string][] {
		return this.keys;
	}
}

export const keyHint = new KeyHint();

// Export a convenience function for adding keys
export const addKeyHint = (key: KeyCombination, hint: string): void => {
	keyHint.addKey(key, hint);
};
export const removeKeyHint = (key: KeyCombination): void => {
	keyHint.removeKey(key);
};
