<script lang="ts">
	import { type } from '@tauri-apps/plugin-os';
	import { t } from 'svelte-i18n-lingui';
	import type { KeyCombination } from '$lib/keyboard';
	import { onMount } from 'svelte';

	let { keyCombination }: { keyCombination: KeyCombination } = $props();

	let osName = $state("default");

	onMount(() => osName = type())

	let keySymbols: Record<string, Record<string, string>> = $derived({
		darwin: {
			ctrl: '⌃',
			control: '⌃',
			cmd: '⌘',
			command: '⌘',
			alt: '⌥',
			option: '⌥',
			shift: '⇧',
			enter: '↩',
			return: '↩',
			backspace: '⌫',
			delete: '⌦',
			escape: $t`Esc`,
			arrowup: '↑',
			arrowdown: '↓',
			arrowleft: '←',
			arrowright: '→',
			space: $t`Space`
		},
		default: {
			ctrl: $t`Ctrl`,
			control: $t`Ctrl`,
			cmd: $t`Win`,
			command: $t`Win`,
			alt: $t`Alt`,
			option: $t`Alt`,
			shift: $t`Shift`,
			enter: $t`Enter`,
			return: $t`Enter`,
			backspace: 'Backspace',
			delete: $t`Delete`,
			escape: $t`Esc`,
			arrowup: '↑',
			arrowdown: '↓',
			arrowleft: '←',
			arrowright: '→',
			space: $t`Space`
		}
	});

	let symbolMap = $derived(osName === 'macos' ? keySymbols.darwin : keySymbols.default);
	let formatted = $derived.by(() => {
		return keyCombination.toLowerCase().split('+').map(key => {
			const trimmedKey = key.trim();
			return symbolMap[trimmedKey] || trimmedKey.toUpperCase();
		});
	});
</script>

<kbd class="kbd kbd-sm">{formatted}</kbd>
