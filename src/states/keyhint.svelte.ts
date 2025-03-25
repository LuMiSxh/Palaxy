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
		const existingIndex = this.keys.findIndex((k) => k[0] === key);

		if (existingIndex >= 0) {
			// If exists, override the hint
			this.keys = this.keys.map((k) => (k[0] === key ? [key, hint] : k));
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
		this.keys = this.keys.filter(([k]) => k !== key);
		return this;
	}

	/**
	 * Adds multiple key combinations with their hints. Returns a function to remove the added key combinations.
	 * @param keyhints An array of key combinations and their hints
	 * @param ignore An array of key combinations to ignore when re-applying the previous key combinations
	 * @returns A function to remove the added key combinations
	 */
	smartAdd(keyhints: Parameters<typeof this.addKey>[], ignore: KeyCombination[] | undefined = undefined): () => void {
		// Get all current key hints
		const currentKeys = this.keys;

		// Add the new key hints
		keyhints.forEach(([key, hint]) => this.addKey(key, hint));

		// Return a function to remove the added key hints and restore the previous ones
		return () => {
			if (ignore !== undefined) {
				this.keys = currentKeys.filter(([key]) => !ignore.includes(key));
			} else {
				this.keys = currentKeys;
			}
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

/**
 * Adds key hints to an element when focused. And removes them when blurred.
 * @param node
 * @param data
 */
export function handleKeyHint(
	node: HTMLElement,
	data: { keys: Parameters<typeof keyHint.addKey>[]; reset?: boolean }
) {
	let oldKeyHints = keyHint.get();

	const addHints = () =>
		data.keys.forEach((key) => {
			oldKeyHints = keyHint.get();
			keyHint.addKey(...key);
		});

	const removeHints = () => {
		if (data.reset) {
			keyHint.set(oldKeyHints);
		} else {
			data.keys.forEach(([key]) => keyHint.removeKey(key));
		}
	};

	node.addEventListener('focus', addHints);
	node.addEventListener('blur', removeHints);

	return {
		destroy() {
			removeHints();
			node.removeEventListener('focus', addHints);
			node.removeEventListener('blur', removeHints);
		},
		update(newKeys: Parameters<typeof keyHint.addKey>[]) {
			removeHints();
			data.keys = newKeys;
			if (document.activeElement === node) {
				addHints();
			}
		}
	};
}
