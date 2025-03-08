<script lang="ts">
	import { onMount } from 'svelte';
	import convState, { stepState } from '$states/converter.svelte';
	import { IconCircleMinus, IconCirclePlus, IconExclamationCircle } from '@tabler/icons-svelte';
	import { commands } from '$types';
	import { wrapper } from '$lib/utils';

	let positives: string[] = $state([]);
	let negatives: string[] = $state([]);
	let suggestions: string[] = $state([]);

	onMount(async () => {
		// Disable the next button, enable the previous button
		stepState.disableNext = true;
		stepState.disablePrev = false;

		const result = await wrapper(commands.convAnalyze());
		if (result !== null && result.payload !== null) {
			convState.bundleRecommendation = result.payload.flag;
			positives = result.payload.positive;
			negatives = result.payload.negative;
			suggestions = result.payload.suggest;

			// When there are no negatives, enable the next button
			if (negatives.length === 0) {
				stepState.disableNext = false;
			}
		}
	});
</script>

{#snippet item(message: string, Icon: any, bg: string, txt: string)}
	<li class="list-row items-center rounded {bg}">
		<Icon class={txt} />
		<span class="list-col-grow text-md {txt}">
			{message}
		</span>
	</li>
{/snippet}

<ul class="list">
	{#each negatives as negative, i (i)}
		{@render item(negative, IconCircleMinus, 'bg-error/70', 'text-error-content')}
	{/each}
	{#each positives as positive, i (i)}
		{@render item(positive, IconCirclePlus, 'bg-success/70', 'text-success-content')}
	{/each}
	{#each suggestions as suggestion, i (i)}
		{@render item(suggestion, IconExclamationCircle, 'bg-info/70', 'text-info-content')}
	{/each}
</ul>
