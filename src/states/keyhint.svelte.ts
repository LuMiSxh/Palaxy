import type { KeyCombination } from '$types/keys';
import { untrack } from 'svelte';

type Scope = {
	keys: [KeyCombination, string][];
	exclusive: boolean;
};

class KeyHintState {
	// We store hints in specific 'scopes' (layers).
	// This allows multiple components to add keys without overwriting or needing to save/restore snapshots.
	private scopes = $state<Record<string, Scope>>({});

	/**
	 * Register keys.
	 * @param exclusive If true, hides hints from all non-exclusive scopes while active.
	 */
	register(keys: [KeyCombination, string][], exclusive = false): () => void {
		const id = Math.random().toString(36).slice(2);
		untrack(() => {
			this.scopes = { ...this.scopes, [id]: { keys, exclusive } };
		});
		return () => {
			untrack(() => {
				const { [id]: _, ...rest } = this.scopes;
				this.scopes = rest;
			});
		};
	}

	/**
	 * Legacy support for global/static keys (like Layout).
	 * These go into a 'global' scope.
	 */
	addKey(key: KeyCombination, hint: string) {
		untrack(() => {
			const currentScope = this.scopes['global'] || { keys: [], exclusive: false };
			const otherKeys = currentScope.keys.filter((k) => k[0] !== key);

			this.scopes = {
				...this.scopes,
				global: {
					keys: [...otherKeys, [key, hint]],
					exclusive: false,
				},
			};
		});
	}

	removeKey(key: KeyCombination) {
		untrack(() => {
			const currentScope = this.scopes['global'];
			if (!currentScope) return;

			this.scopes = {
				...this.scopes,
				global: {
					keys: currentScope.keys.filter((k) => k[0] !== key),
					exclusive: false,
				},
			};
		});
	}

	clear() {
		untrack(() => {
			this.scopes = {};
		});
	}

	/**
	 * Flattens all scopes into a single list for the UI.
	 * Later registrations (like ActionHub) naturally appear at the end
	 * if we iterate object keys, or we can just merge them.
	 */
	get(): [KeyCombination, string][] {
		const allScopes = Object.values(this.scopes);

		// Check if ANY active scope is exclusive
		const hasExclusive = allScopes.some((s) => s.exclusive);

		const merged = new Map<string, string>();

		allScopes.forEach((scope) => {
			// If we are in exclusive mode, skip any scope that isn't exclusive
			if (hasExclusive && !scope.exclusive) return;

			scope.keys.forEach(([key, label]) => merged.set(key, label));
		});

		return Array.from(merged.entries()) as [KeyCombination, string][];
	}
}

export const keyHint = new KeyHintState();

let activeCleanup: (() => void) | null = null;

function handleFocusIn(event: FocusEvent) {
	// 1. Always clean up the previous element's hints first
	if (activeCleanup) {
		activeCleanup();
		activeCleanup = null;
	}

	const target = event.target as HTMLElement;
	// Check if target exists and has the attribute
	if (!target || !target.getAttribute) return;

	const attribute = target.getAttribute('data-keyhint');
	if (!attribute) return;

	// 2. Parse the attribute
	// Supports single: "enter;Invoke"
	// Supports multiple: "enter;Invoke|esc;Cancel"
	const hints: [KeyCombination, string][] = attribute.split('|').map((part) => {
		const [key, label] = part.split(';');
		return [key.trim() as KeyCombination, label?.trim() || ''];
	});

	if (hints.length > 0) {
		// 3. Register and save cleanup
		activeCleanup = keyHint.register(hints);
	}
}

function handleFocusOut(event: FocusEvent) {
	// Always clean up when an element with data-keyhint loses focus
	// This ensures hints are removed when navigating between pages/steps
	if (activeCleanup) {
		activeCleanup();
		activeCleanup = null;
	}
}

/**
 * Manually clear the active keyhint cleanup.
 * Useful when navigating between pages/steps.
 */
export function clearActiveKeyHint() {
	if (activeCleanup) {
		activeCleanup();
		activeCleanup = null;
	}
}

/**
 * Mounts the global event listeners for data-keyhint attributes.
 * Call this once in your root layout onMount.
 */
export function mountGlobalKeyHintListener() {
	if (typeof window !== 'undefined') {
		document.addEventListener('focusin', handleFocusIn);
		document.addEventListener('focusout', handleFocusOut);

		return () => {
			document.removeEventListener('focusin', handleFocusIn);
			document.removeEventListener('focusout', handleFocusOut);
		};
	}
	return () => {};
}

/**
 * Action to handle key hints on DOM elements focus/blur
 */
export function handleKeyHint(
	node: HTMLElement,
	data: { keys: [KeyCombination, string][]; reset?: boolean }
) {
	let unregister: (() => void) | null = null;

	const addHints = () => {
		if (unregister) unregister(); // Safety check
		unregister = keyHint.register(data.keys);
	};

	const removeHints = () => {
		if (unregister) {
			unregister();
			unregister = null;
		}
	};

	node.addEventListener('focus', addHints);
	node.addEventListener('blur', removeHints);

	// If element is already focused when action mounts
	if (document.activeElement === node) {
		addHints();
	}

	return {
		destroy() {
			removeHints();
			node.removeEventListener('focus', addHints);
			node.removeEventListener('blur', removeHints);
		},
		update(newData: typeof data) {
			removeHints();
			data = newData;
			if (document.activeElement === node) {
				addHints();
			}
		},
	};
}
