<script lang="ts">
	import { type } from '@tauri-apps/plugin-os';
	import type { KeyCombination } from '$types/keys';

	let { keyCombination }: { keyCombination: KeyCombination } = $props();

	const osName = type();

	const keySymbols: Record<string, Record<string, string>> = {
		macos: {
			ctrl: '⌃',
			control: '⌃',
			cmd: '⌘',
			command: '⌘',
			alt: '⌥',
			option: '⌥',
			shift: '⇧',
			tab: '⇥',
			enter: '↩',
			return: '↩',
			backspace: '⌫',
			delete: '⌦',
			escape: 'Esc',
			arrowup: '↑',
			arrowdown: '↓',
			arrowleft: '←',
			arrowright: '→',
			space: '␣',
			plus: '+',
			minus: '-',
		},
		default: {
			ctrl: 'Ctrl',
			control: 'Ctrl',
			cmd: 'Win',
			command: 'Win',
			alt: 'Alt',
			option: 'Alt',
			shift: '⇧',
			tab: '⭾',
			enter: '⏎',
			return: '⏎',
			backspace: '⌫',
			delete: 'Del',
			escape: 'Esc',
			arrowup: '↑',
			arrowdown: '↓',
			arrowleft: '←',
			arrowright: '→',
			space: '␣',
			plus: '+',
			minus: '-',
		},
	};

	let symbolMap = $derived(keySymbols[osName] || keySymbols.default);
	let formatted = $derived.by(() => {
		return keyCombination
			.toLowerCase()
			.split('+')
			.map((key) => {
				const trimmedKey = key.trim();
				return symbolMap[trimmedKey] || trimmedKey.toUpperCase();
			})
			.join(' ');
	});
</script>

<kbd class="kbd kbd-sm select-none">{formatted}</kbd>
