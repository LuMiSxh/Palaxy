<script lang="ts">
	import { keyHint } from '$states/keyhint.svelte';
	import KeyComboDisplay from '$components/KeyComboDisplay.svelte';
	import { cubicOut } from 'svelte/easing';
	import type { KeyCombination } from '$types/keys';

	function getKeyTypePriority(key: string): number {
		const k = key.toLowerCase();
		if (k.includes('space')) return 0;
		if (k.includes('arrow')) return 10;
		if (k.includes('tab')) return 11;
		if (k.includes('enter')) return 20;
		if (k.includes('esc')) return 30;
		return 50;
	}

	// Custom "Glass Motion" Transition
	// Layout: Width + Margin (prevents jumping)
	// Visual: Slide + Scale + Blur (the "glass" feel)
	function glassKeyTransition(node: Element, { duration = 300, delay = 0 }) {
		const style = getComputedStyle(node);
		const targetWidth = parseFloat(style.width);
		const targetMarginRight = parseFloat(style.marginRight);
		const targetOpacity = +style.opacity;

		return {
			duration,
			delay,
			css: (t: number) => {
				const eased = cubicOut(t);

				// 1. Layout: Collapse width & margin so neighbors move smoothly
				const curWidth = targetWidth * eased;
				const curMargin = targetMarginRight * eased;

				// 2. Motion calculation
				const translate = (1 - eased) * -20; // Slide from -20px left
				const scale = 0.85 + 0.15 * eased; // Scale from 85% to 100%
				const blur = (1 - eased) * 5; // Blur from 5px to 0px

				return `
                    width: ${curWidth}px;
                    margin-right: ${curMargin}px;

                    opacity: ${targetOpacity * eased};
                    transform: translateX(${translate}px) scale(${scale});
                    filter: blur(${blur}px);

                    overflow: hidden;
                    white-space: nowrap;
                `;
			},
		};
	}

	let groupedHints = $derived.by(() => {
		const rawHints = keyHint.get();
		const groups: Record<string, string[]> = {};

		for (const [key, desc] of rawHints) {
			if (!groups[desc]) groups[desc] = [];
			groups[desc].push(key);
		}

		return Object.entries(groups)
			.map(([desc, keys]) => {
				keys.sort((a, b) => a.length - b.length);
				return {
					description: desc,
					keys: keys as KeyCombination[],
					priority: getKeyTypePriority(keys[0]),
				};
			})
			.sort((a, b) => a.priority - b.priority);
	});
</script>

<ul class="flex h-full w-full items-center justify-center" draggable="false">
	{#each groupedHints as group (group.description)}
		<li class="mr-4 flex items-center gap-2" transition:glassKeyTransition={{ duration: 300 }}>
			<div class="flex items-center gap-1">
				{#each group.keys as key, k (k)}
					<KeyComboDisplay keyCombination={key} />
					{#if k < group.keys.length - 1}
						<span class="text-muted text-xs font-bold opacity-50">/</span>
					{/if}
				{/each}
			</div>

			<span class="text-muted text-sm font-medium whitespace-nowrap">
				{group.description}
			</span>

			<div class="divider bg-waku-border/50 ml-2 h-4 w-px"></div>
		</li>
	{/each}
</ul>

<style>
	/* Hide divider on the last visible child */
	li:last-child .divider {
		display: none;
	}

	/* Remove margin on the last item to keep true centering */
	li:last-child {
		margin-right: 0 !important;
	}
</style>
