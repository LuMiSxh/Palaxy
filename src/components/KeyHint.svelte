<script lang="ts">
	import { keyHint } from "$states/keyhint.svelte"
	import KeyComboDisplay from '$components/KeyComboDisplay.svelte';

	// Define key type priority order
	function getKeyTypePriority(key: string): number {
		if (key.includes('space')) return 0;
		if (key.includes('esc')) return 1;
		if (key.includes('enter')) return 2;
		if (key.includes('arrow') || key.includes('up') || key.includes('down') ||
			key.includes('left') || key.includes('right')) return 3;
		if (key.includes('tab')) return 4;
		if (key.includes('ctrl') || key.includes('alt') ||
			key.includes('shift') || key.includes('cmd')) return 5;
		// Function keys
		if (/f[0-9]+/.test(key)) return 6;
		// Numbers
		if (/^[0-9]$/.test(key)) return 7;
		// Letters (alphabetical)
		if (/^[a-z]$/.test(key)) return 8;

		return 9; // Other keys
	}

	// Sort the key hints
	let sorted = $derived.by(() => {
		return [...keyHint.get()].sort((a, b) => {
			return getKeyTypePriority(a[0]) - getKeyTypePriority(b[0]);
		});
	})
</script>

<ul class="flex gap-2 w-full h-full">
	{#each sorted as [ key, description ], i}
		<li class="flex gap-1 items-center">
			<KeyComboDisplay keyCombination={key} />
			<span class="text-sm">{description}</span>
		</li>
		{#if i < sorted.length - 1}
			<div class="divider-horizontal mx-1!"></div>
		{/if}
	{/each}
</ul>
